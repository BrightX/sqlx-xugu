use crate::error::Error;
use crate::io::AsyncStreamExt;
use crate::protocol::message::*;
use crate::protocol::statement::{Execute as StatementExecute, Prepare, StmtClose};
use crate::protocol::text::{ColumnFlags, OkPacket, Query};
use crate::protocol::ServerContext;
use crate::statement::{XuguStatement, XuguStatementMetadata};
use crate::{
    Xugu, XuguArguments, XuguConnection, XuguDatabaseError, XuguQueryResult, XuguRow, XuguTypeInfo,
};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use log::Level;
use sqlx_core::describe::Describe;
use sqlx_core::executor::{Execute, Executor};
use sqlx_core::logger::QueryLogger;
use sqlx_core::{try_stream, Either, HashMap};
use std::{borrow::Cow, pin::pin, sync::Arc};

impl XuguConnection {
    async fn prepare_statement<'c>(
        &mut self,
        sql: &str,
    ) -> Result<(u32, XuguStatementMetadata), Error> {
        // flush and wait until we are re-ready
        self.wait_until_ready().await?;

        let id = self.inner.gen_st_id();
        self.inner
            .stream
            .send_packet(Prepare {
                query: sql,
                con_obj_name: &self.inner.con_obj_name,
                st_id: id,
            })
            .await?;

        let mut error = None;
        let mut columns = Vec::new();
        let mut column_names = HashMap::new();
        let mut params = Vec::new();

        loop {
            let message: ReceivedMessage = self.inner.stream.recv().await?;
            let cnt = ServerContext::new(self.inner.stream.server_version);
            match message.format {
                BackendMessageFormat::ErrorResponse => {
                    let err: ErrorResponse = message.decode(&mut self.inner.stream, cnt).await?;
                    error = Some(err.error);
                }
                BackendMessageFormat::MessageResponse => {
                    // 读到服务器端返回消息用对话框抛出
                    // 警告和信息
                    let notice: MessageResponse =
                        message.decode(&mut self.inner.stream, cnt).await?;
                    let (log_level, tracing_level) = (Level::Info, tracing::Level::INFO);
                    let log_is_enabled = log::log_enabled!(
                        target: "sqlx::xugu::notice",
                        log_level
                    ) || sqlx_core::private_tracing_dynamic_enabled!(
                        target: "sqlx::xugu::notice",
                        tracing_level
                    );
                    if log_is_enabled {
                        sqlx_core::private_tracing_dynamic_event!(
                            target: "sqlx::xugu::notice",
                            tracing_level,
                            message = notice.msg
                        );
                    }
                }
                BackendMessageFormat::ReadyForQuery => {
                    let _: ReadyForQuery = message.decode(&mut self.inner.stream, cnt).await?;
                    break;
                }
                BackendMessageFormat::RowDescription => {
                    let row_columns: RowDescription =
                        message.decode(&mut self.inner.stream, cnt).await?;
                    (columns, column_names) = row_columns.convert_columns()?;
                }
                BackendMessageFormat::ParameterDescription => {
                    let param_def: ParameterDescription =
                        message.decode(&mut self.inner.stream, cnt).await?;
                    params = param_def.params;
                }
                _ => {
                    break;
                }
            }
        }

        if let Some(err) = error {
            return Err(Error::Database(Box::new(XuguDatabaseError::from_str(&err))));
        }

        let metadata = XuguStatementMetadata {
            parameters: Arc::new(params),
            columns: Arc::new(columns),
            column_names: Arc::new(column_names),
        };

        Ok((id, metadata))
    }

    async fn get_or_prepare_statement<'c>(
        &mut self,
        sql: &str,
    ) -> Result<(u32, XuguStatementMetadata), Error> {
        if let Some(statement) = self.inner.cache_statement.get_mut(sql) {
            // <XuguStatementMetadata> is internally reference-counted
            return Ok((*statement).clone());
        }

        let (id, metadata) = self.prepare_statement(sql).await?;

        // in case of the cache being full, close the least recently used statement
        if let Some((id, _)) = self
            .inner
            .cache_statement
            .insert(sql, (id, metadata.clone()))
        {
            // flush and wait until we are re-ready
            self.wait_until_ready().await?;
            self.inner
                .stream
                .send_packet(StmtClose {
                    con_obj_name: &self.inner.con_obj_name,
                    st_id: id,
                })
                .await?;

            // for StmtClose
            let _ok: OkPacket = self.inner.stream.recv().await?;
        }

        Ok((id, metadata))
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `sql`:
    /// * `arguments`:
    /// * `persistent`: sql 语句是否需要被缓存
    ///
    #[allow(clippy::needless_lifetimes)]
    pub(crate) async fn run<'e, 'c: 'e, 'q: 'e>(
        &'c mut self,
        sql: &'q str,
        arguments: Option<XuguArguments<'q>>,
        persistent: bool,
    ) -> Result<impl Stream<Item = Result<Either<XuguQueryResult, XuguRow>, Error>> + 'e, Error>
    {
        let mut logger = QueryLogger::new(sql, self.inner.log_settings.clone());

        self.wait_until_ready().await?;

        // make a slot for the shared column data
        // as long as a reference to a row is not held past one iteration, this enables us
        // to re-use this memory freely between result sets
        let (mut column_names, mut columns, mut needs_metadata) = if let Some(arguments) = arguments
        {
            if persistent && self.inner.cache_statement.is_enabled() {
                let (id, metadata) = self.get_or_prepare_statement(sql).await?;

                self.inner
                    .stream
                    .send_packet(StatementExecute {
                        con_obj_name: &self.inner.con_obj_name,
                        st_id: id,
                        arguments: &arguments,
                        params: &metadata.parameters,
                    })
                    .await?;

                let needs_metadata = metadata.column_names.is_empty();
                (metadata.column_names, metadata.columns, needs_metadata)
            } else {
                let (id, metadata) = self.prepare_statement(sql).await?;

                self.inner
                    .stream
                    .send_packet(StatementExecute {
                        con_obj_name: &self.inner.con_obj_name,
                        st_id: id,
                        arguments: &arguments,
                        params: &metadata.parameters,
                    })
                    .await?;

                self.inner
                    .stream
                    .send_packet(StmtClose {
                        con_obj_name: &self.inner.con_obj_name,
                        st_id: id,
                    })
                    .await?;
                // for StmtClose
                self.inner.pending_ready_for_query_count += 1;

                let needs_metadata = metadata.column_names.is_empty();
                (metadata.column_names, metadata.columns, needs_metadata)
            }
        } else {
            self.inner.stream.send_packet(Query(sql)).await?;

            (Arc::default(), Arc::default(), true)
        };

        self.inner.pending_ready_for_query_count += 1;

        let mut error = None;

        let mut num_columns = 0;

        Ok(try_stream! {
            loop {
                let message: ReceivedMessage = self.inner.stream.recv().await?;
                let cnt = ServerContext::new(self.inner.stream.server_version);
                match message.format {
                    BackendMessageFormat::ErrorResponse => {
                        let err: ErrorResponse = message.decode(&mut self.inner.stream, cnt).await?;
                        error = Some(err.error);
                    },
                    BackendMessageFormat::MessageResponse => {
                        // 读到服务器端返回消息用对话框抛出
                        // 警告和信息
                        let notice: MessageResponse = message.decode(&mut self.inner.stream, cnt).await?;
                        let (log_level, tracing_level) = (Level::Info, tracing::Level::INFO);
                        let log_is_enabled = log::log_enabled!(
                            target: "sqlx::xugu::notice",
                            log_level
                        ) || sqlx_core::private_tracing_dynamic_enabled!(
                            target: "sqlx::xugu::notice",
                            tracing_level
                        );
                        if log_is_enabled {
                            sqlx_core::private_tracing_dynamic_event!(
                                target: "sqlx::xugu::notice",
                                tracing_level,
                                message = notice.msg
                            );
                        }
                    },
                    BackendMessageFormat::ReadyForQuery => {
                        //命令结束 / 错误结束
                        let _: ReadyForQuery = message.decode(&mut self.inner.stream, cnt).await?;
                        self.handle_ready_for_query().await?;
                        break;
                    },
                    BackendMessageFormat::InsertResponse => {
                        let res: InsertResponse = message.decode(&mut self.inner.stream, cnt).await?;
                        let rows_affected = 1;
                        logger.increase_rows_affected(rows_affected);
                        let done = XuguQueryResult {
                            rows_affected,
                            last_insert_id: Some(res.rowid),
                        };
                        r#yield!(Either::Left(done));
                    },
                    BackendMessageFormat::DeleteResponse => {
                        let res: DeleteResponse = message.decode(&mut self.inner.stream, cnt).await?;
                        let rows_affected = res.rows_affected as u64;
                        logger.increase_rows_affected(rows_affected);
                        let done = XuguQueryResult {
                            rows_affected,
                            last_insert_id: None,
                        };
                        r#yield!(Either::Left(done));
                    },
                    BackendMessageFormat::UpdateResponse => {
                        let res: UpdateResponse = message.decode(&mut self.inner.stream, cnt).await?;
                        let rows_affected = res.rows_affected as u64;
                        logger.increase_rows_affected(rows_affected);
                        let done = XuguQueryResult {
                            rows_affected,
                            last_insert_id: None,
                        };
                        r#yield!(Either::Left(done));
                    },
                    BackendMessageFormat::RowDescription => {
                        // 接收列数据
                        let row_columns: RowDescription = message.decode(&mut self.inner.stream, cnt).await?;
                        num_columns = row_columns.fields.len();
                        self.inner.last_num_columns = num_columns;
                        if needs_metadata {
                            let (columns_c, column_names_c) = row_columns.convert_columns()?;
                            columns = Arc::new(columns_c);
                            column_names = Arc::new(column_names_c);
                        } else {
                            // next time we hit here, it'll be a new result set and we'll need the
                            // full metadata
                            needs_metadata = true;
                        }
                    },
                    BackendMessageFormat::ParameterDescription => {
                        let _: ParameterDescription = message.decode(&mut self.inner.stream, cnt).await?;
                    },
                    BackendMessageFormat::DataRow => {
                        // 接收行数据
                        let _: DataRow = message.decode(&mut self.inner.stream, cnt).await?;
                        let mut row = Vec::with_capacity(num_columns);
                        for _ in 0..num_columns {
                            let len = self.inner.stream.read_i32().await?;
                            let buf = self.inner.stream.read_bytes(len as usize).await?;
                            row.push(buf);
                        }
                        let row = Arc::new(row);

                        let v = Either::Right(XuguRow {
                            row,
                            columns: Arc::clone(&columns),
                            column_names: Arc::clone(&column_names),
                        });

                        logger.increment_rows_returned();

                        r#yield!(v);
                    }
                }
            }

            if let Some(err) = error {
                return Err(Error::Database(Box::new(XuguDatabaseError::from_str(&err))));
            }

            return Ok(());
        })
    }
}

impl<'c> Executor<'c> for &'c mut XuguConnection {
    type Database = Xugu;

    /// 执行多个查询，并将生成的结果作为每个查询的流返回。
    fn fetch_many<'e, 'q, E>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<XuguQueryResult, XuguRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
        'q: 'e,
        E: 'q,
    {
        let sql = query.sql();
        let arguments = query.take_arguments().map_err(Error::Encode);
        let persistent = query.persistent();

        Box::pin(try_stream! {
            let arguments = arguments?;
            let mut s = pin!(self.run(sql, arguments, persistent).await?);

            while let Some(v) = s.try_next().await? {
                r#yield!(v);
            }

            Ok(())
        })
    }

    /// 执行查询并最多返回一行。
    fn fetch_optional<'e, 'q, E>(self, query: E) -> BoxFuture<'e, Result<Option<XuguRow>, Error>>
    where
        'c: 'e,
        E: Execute<'q, Self::Database>,
        'q: 'e,
        E: 'q,
    {
        let mut s = self.fetch_many(query);

        Box::pin(async move {
            while let Some(v) = s.try_next().await? {
                if let Either::Right(r) = v {
                    return Ok(Some(r));
                }
            }

            Ok(None)
        })
    }

    /// 准备 SQL 查询，其中包含参数类型信息，以检查有关其参数和结果的类型信息。
    ///
    /// 只有某些数据库驱动程序（PostgreSQL、MSSQL）可以利用此额外信息来影响参数类型推断。
    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        _parameters: &'e [XuguTypeInfo],
    ) -> BoxFuture<'e, Result<XuguStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            self.wait_until_ready().await?;

            let metadata = if self.inner.cache_statement.is_enabled() {
                self.get_or_prepare_statement(sql).await?.1
            } else {
                let (id, metadata) = self.prepare_statement(sql).await?;

                self.inner
                    .stream
                    .send_packet(StmtClose {
                        con_obj_name: &self.inner.con_obj_name,
                        st_id: id,
                    })
                    .await?;
                // for StmtClose
                let _ok: OkPacket = self.inner.stream.recv().await?;

                metadata
            };

            Ok(XuguStatement {
                sql: Cow::Borrowed(sql),
                // metadata has internal Arcs for expensive data structures
                metadata: metadata.clone(),
            })
        })
    }

    /// 描述有关其参数和结果的 SQL 查询和返回类型信息。
    ///
    /// 查询宏中的编译时验证使用它来支持其类型推断。
    #[doc(hidden)]
    fn describe<'e, 'q: 'e>(self, sql: &'q str) -> BoxFuture<'e, Result<Describe<Xugu>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            self.wait_until_ready().await?;

            let (id, metadata) = self.prepare_statement(sql).await?;

            self.inner
                .stream
                .send_packet(StmtClose {
                    con_obj_name: &self.inner.con_obj_name,
                    st_id: id,
                })
                .await?;
            // for StmtClose
            let _ok: OkPacket = self.inner.stream.recv().await?;

            let columns = (*metadata.columns).clone();

            let nullable = columns
                .iter()
                .map(|col| {
                    col.flags
                        .map(|flags| !flags.contains(ColumnFlags::NOT_NULL))
                })
                .collect();

            Ok(Describe {
                parameters: Some(Either::Right(metadata.parameters.len())),
                columns,
                nullable,
            })
        })
    }
}
