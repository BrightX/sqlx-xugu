use super::XuguStream;
use crate::error::Error;
use crate::io::AsyncStreamExt;
use crate::protocol::statement::{Execute as StatementExecute, Prepare, PrepareOk, StmtClose};
use crate::protocol::text::{ColumnDefinition, ColumnFlags, Query};
use crate::statement::{XuguStatement, XuguStatementMetadata};
use crate::{
    Xugu, XuguArguments, XuguColumn, XuguConnection, XuguDatabaseError, XuguQueryResult, XuguRow,
    XuguTypeInfo,
};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_core::Stream;
use futures_util::TryStreamExt;
use sqlx_core::describe::Describe;
use sqlx_core::executor::{Execute, Executor};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::logger::QueryLogger;
use sqlx_core::{err_protocol, try_stream, Either, HashMap};
use std::{borrow::Cow, pin::pin, sync::Arc};

impl XuguConnection {
    async fn prepare_statement<'c>(
        &mut self,
        sql: &str,
    ) -> Result<(u32, XuguStatementMetadata), Error> {
        let id = self.inner.gen_st_id();
        self.inner
            .stream
            .send_packet(Prepare {
                query: sql,
                con_obj_name: &self.inner.con_obj_name,
                st_id: id,
            })
            .await?;

        let ok: PrepareOk = self.inner.stream.recv().await?;

        let mut columns = Vec::new();

        let column_names = if !ok.columns.is_empty() {
            let num_columns = ok.columns.len();

            let mut column_names = HashMap::with_capacity(num_columns);

            for ordinal in 0..num_columns {
                let def = &ok.columns[ordinal];

                let column = recv_next_result_column(&def, ordinal)?;

                column_names.insert(column.name.clone(), ordinal);
                // 列名不区分大小写，将大写和小写列名都加入
                column_names.insert(column.name.to_uppercase().into(), ordinal);
                column_names.insert(column.name.to_lowercase().into(), ordinal);
                columns.push(column);
            }

            column_names
        } else {
            Default::default()
        };

        let metadata = XuguStatementMetadata {
            parameters: Arc::new(ok.params),
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
            self.inner
                .stream
                .send_packet(StmtClose {
                    con_obj_name: &self.inner.con_obj_name,
                    st_id: id,
                })
                .await?;
        }

        Ok((id, metadata))
    }

    async fn next_byte(&mut self) -> Result<u8, Error> {
        self.inner.stream.read_u8().await
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

        // self.inner.stream.wait_until_ready().await?;
        // self.inner.stream.waiting.push_back(Waiting::Result);

        Ok(try_stream! {
            // make a slot for the shared column data
            // as long as a reference to a row is not held past one iteration, this enables us
            // to re-use this memory freely between result sets
            let mut columns = Arc::new(Vec::new());

            let (mut column_names, mut needs_metadata) = if let Some(arguments) = arguments {
                if persistent && self.inner.cache_statement.is_enabled() {
                    let (id, metadata) = self
                        .get_or_prepare_statement(sql)
                        .await?;

                    self.inner.stream
                        .send_packet(StatementExecute {
                            con_obj_name: &self.inner.con_obj_name, st_id: id,
                            arguments: &arguments,
                            params: &metadata.parameters,
                        })
                        .await?;

                    let needs_metadata = metadata.column_names.is_empty();
                    (metadata.column_names, needs_metadata)
                } else {
                    let (id, metadata) = self
                        .prepare_statement(sql)
                        .await?;

                    self.inner.stream
                        .send_packet(StatementExecute {
                            con_obj_name: &self.inner.con_obj_name, st_id: id,
                            arguments: &arguments,
                            params: &metadata.parameters,
                        })
                        .await?;

                    self.inner.stream.send_packet(StmtClose {con_obj_name: &self.inner.con_obj_name, st_id: id }).await?;

                    let needs_metadata = metadata.column_names.is_empty();
                    (metadata.column_names, needs_metadata)
                }
            } else {
                self.inner.stream.send_packet(Query(sql)).await?;

                (Arc::default(), true)
            };

            let mut warnings = Vec::new();
            let mut error = None;

            let mut num_columns = 0;

            let mut bt = self.next_byte().await?;
            loop {
                match bt {
                    b'E' | b'F' => {
                        let err = self.inner.stream.read_str().await?;
                        error = Some(err);
                        bt = self.next_byte().await?;
                    },
                    b'W' | b'M' => {
                        // 读到服务器端返回消息用对话框抛出
                        // 警告和信息
                        let warn = self.inner.stream.read_str().await?;
                        warnings.push(warn);
                        bt = self.next_byte().await?;
                    },
                    b'K' | b'<' => {
                        //命令结束 / 错误结束
                        break;
                    },
                    b'U' => {
                        // 批处理执行次数 / 批量更新次数
                        let up_int = self.inner.stream.read_i32().await?;
                        println!("serverBatchCount / updateCount = {}", up_int);

                        let rows_affected = up_int as u64;
                        logger.increase_rows_affected(rows_affected);
                        let done = XuguQueryResult {
                            rows_affected,
                            last_insert_id: None,
                        };

                        bt = self.next_byte().await?;
                        r#yield!(Either::Left(done));
                    },
                    b'D' => {
                        // 批处理执行次数 / 批量更新次数
                        let del_int = self.inner.stream.read_i32().await?;
                        println!("serverBatchCount / updateCount = {}", del_int);

                        let rows_affected = del_int as u64;
                        logger.increase_rows_affected(rows_affected);
                        let done = XuguQueryResult {
                            rows_affected,
                            last_insert_id: None,
                        };

                        bt = self.next_byte().await?;
                        r#yield!(Either::Left(done));
                    },
                    b'S' => {
                        let lob_id = self.inner.stream.read_i64().await?;
                        // getParamLob(lobId)
                        // sendLob(paramLob)
                        println!("lob_id: {}", lob_id);
                        bt = self.next_byte().await?;
                    },
                    b'I' => {
                        let rowid = self.inner.stream.read_str().await?;

                        let rows_affected = 1;
                        logger.increase_rows_affected(rows_affected);
                        let done = XuguQueryResult {
                            rows_affected,
                            last_insert_id: Some(rowid),
                        };
                        if self.inner.stream.server_version >= 302 {
                            let col_no = self.inner.stream.read_i32().await?;
                            if col_no >= 0 {
                                let identity = self.inner.stream.read_bytes(8).await?;
                                println!("identity: {:?}", identity);
                            }
                        }

                        bt = self.next_byte().await?;
                        r#yield!(Either::Left(done));
                    },
                    b'L' => {
                        // filename
                        let filename = self.inner.stream.read_str().await?;
                        // sendFile(filename);
                        println!("filename: {}", filename);
                        bt = self.next_byte().await?;
                    },
                    b'P' => {
                        // prepare服务器返回参数/过程和函数返回值
                        let no = self.inner.stream.read_i32().await?;
                        let type_id = self.inner.stream.read_i32().await?;
                        let d_len = self.inner.stream.read_i32().await?;
                        println!("no: {}, type: {}, d_len: {}", no, type_id, d_len);
                        if d_len > 0 {
                            let data = self.inner.stream.read_bytes(d_len as usize).await?;
                            println!("dat = {:?}", data);
                        }
                        bt = self.next_byte().await?;
                    },
                    b'O' => {
                        // prepare服务器返回参数/过程和函数返回值
                        let type_id = self.inner.stream.read_i32().await?;
                        let d_len = self.inner.stream.read_i32().await?;
                        println!("type: {}, d_len: {}", type_id, d_len);
                        if d_len > 0 {
                            let data = self.inner.stream.read_bytes(d_len as usize).await?;
                            println!("dat = {:?}", data);
                        }
                        bt = self.next_byte().await?;
                    },
                    b'A' => {
                        // 接收列数据
                        num_columns = self.inner.stream.read_i32().await?;
                        if needs_metadata {
                            column_names = Arc::new(recv_result_metadata(&mut self.inner.stream, num_columns as usize, Arc::make_mut(&mut columns)).await?);
                        } else {
                            // next time we hit here, it'll be a new result set and we'll need the
                            // full metadata
                            needs_metadata = true;

                            recv_result_columns(&mut self.inner.stream, num_columns as usize, Arc::make_mut(&mut columns)).await?;
                        }
                        bt = self.next_byte().await?;
                    },
                    b'R' => {
                        // 接收行数据
                        let mut row = Vec::with_capacity(num_columns as usize);
                        for _ in 0..num_columns {
                            let len = self.inner.stream.read_i32().await?;
                            // println!("len: {}", len);
                            let buf = self.inner.stream.read_bytes(len as usize).await?;
                            // println!("buf = {:?}", buf);
                            row.push(buf);
                        }
                        let row = Arc::new(row);

                        let v = Either::Right(XuguRow {
                            row,
                            columns: Arc::clone(&columns),
                            column_names: Arc::clone(&column_names),
                        });

                        logger.increment_rows_returned();

                        bt = self.next_byte().await?;
                        r#yield!(v);
                    },
                    _ => {
                        return Err(err_protocol!("违反虚谷协议first byte: {}", bt as char));
                    },
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
            // self.inner.stream.wait_until_ready().await?;

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
            // self.inner.stream.wait_until_ready().await?;

            let (id, metadata) = self.prepare_statement(sql).await?;

            self.inner
                .stream
                .send_packet(StmtClose {
                    con_obj_name: &self.inner.con_obj_name,
                    st_id: id,
                })
                .await?;

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

async fn recv_result_columns(
    stream: &mut XuguStream,
    num_columns: usize,
    columns: &mut Vec<XuguColumn>,
) -> Result<(), Error> {
    columns.clear();
    columns.reserve(num_columns);

    for ordinal in 0..num_columns {
        columns.push(recv_next_result_column(&stream.recv().await?, ordinal)?);
    }

    Ok(())
}

fn recv_next_result_column(def: &ColumnDefinition, ordinal: usize) -> Result<XuguColumn, Error> {
    // if the alias is empty, use the alias
    // only then use the name
    let name = match (def.name()?, def.alias()?) {
        (_, alias) if !alias.is_empty() => UStr::new(alias),
        (name, _) => UStr::new(name),
    };

    let type_info = XuguTypeInfo::from_column(def);

    Ok(XuguColumn {
        name,
        type_info,
        ordinal,
        flags: Some(def.flags),
    })
}

async fn recv_result_metadata(
    stream: &mut XuguStream,
    num_columns: usize,
    columns: &mut Vec<XuguColumn>,
) -> Result<HashMap<UStr, usize>, Error> {
    // the result-set metadata is primarily a listing of each output
    // column in the result-set

    let mut column_names = HashMap::with_capacity(num_columns);

    columns.clear();
    columns.reserve(num_columns);

    for ordinal in 0..num_columns {
        let def: ColumnDefinition = stream.recv().await?;

        let column = recv_next_result_column(&def, ordinal)?;

        column_names.insert(column.name.clone(), ordinal);
        columns.push(column);
    }

    Ok(column_names)
}
