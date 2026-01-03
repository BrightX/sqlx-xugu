use self::stream::XuguStream;
use crate::io::AsyncStreamExt;
use crate::protocol::message::*;
use crate::protocol::statement::StmtClose;
use crate::protocol::text::{OkPacket, Ping};
use crate::protocol::ServerContext;
use crate::statement::XuguStatementMetadata;
use crate::{Xugu, XuguConnectOptions, XuguDatabaseError};
use futures_core::future::BoxFuture;
use futures_util::FutureExt;
use log::Level;
use sqlx_core::common::StatementCache;
use sqlx_core::connection::{Connection, LogSettings};
use sqlx_core::transaction::Transaction;
use sqlx_core::{err_protocol, Error};
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};

mod establish;
mod executor;
mod ssl;
mod stream;

pub struct XuguConnection {
    pub(crate) inner: Box<XuguConnectionInner>,
}

pub(crate) struct XuguConnectionInner {
    pub(crate) stream: XuguStream,

    // transaction status
    pub(crate) transaction_depth: usize,
    // status_flags: Status,

    // cache by query string to the statement id and metadata
    cache_statement: StatementCache<(u32, XuguStatementMetadata)>,

    // number of ReadyForQuery messages that we are currently expecting
    pub(crate) pending_ready_for_query_count: usize,
    pub(crate) last_num_columns: usize,

    log_settings: LogSettings,

    st_id_gen: u32,
    con_obj_name: String,
}

impl XuguConnectionInner {
    pub(crate) fn gen_st_id(&mut self) -> u32 {
        self.st_id_gen = self.st_id_gen.wrapping_add(1);
        self.st_id_gen
    }

    pub(super) fn addr_code(&mut self) -> usize {
        let addr = std::ptr::addr_of!(*self) as usize;
        addr
    }
}

impl XuguConnection {
    // will return when the connection is ready for another query
    pub(crate) async fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.inner.stream.write_buffer_mut().is_empty() {
            self.inner.stream.flush().await?;
        }

        let mut num_columns = self.inner.last_num_columns;
        while self.inner.pending_ready_for_query_count > 0 {
            let message: ReceivedMessage = self.inner.stream.recv().await?;
            let cnt = ServerContext::new(self.inner.stream.server_version);
            match message.format {
                BackendMessageFormat::ErrorResponse => {
                    let err: ErrorResponse = message.decode(&mut self.inner.stream, cnt).await?;
                    return Err(Error::Database(Box::new(XuguDatabaseError::from_str(
                        &err.error,
                    ))));
                }
                BackendMessageFormat::MessageResponse => {
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
                BackendMessageFormat::RowDescription => {
                    // 接收列数据
                    let columns: RowDescription =
                        message.decode(&mut self.inner.stream, cnt).await?;
                    num_columns = columns.fields.len();
                    self.inner.last_num_columns = num_columns;
                }
                BackendMessageFormat::DataRow => {
                    // 接收行数据
                    let _: DataRow = message.decode(&mut self.inner.stream, cnt).await?;
                    for _ in 0..num_columns {
                        let len = self.inner.stream.read_i32().await?;
                        let _buf = self.inner.stream.read_bytes(len as usize).await?;
                    }
                }
                BackendMessageFormat::ReadyForQuery => {
                    let _: ReadyForQuery = message.decode(&mut self.inner.stream, cnt).await?;
                    self.handle_ready_for_query().await?;
                }
                BackendMessageFormat::InsertResponse => {
                    let _: InsertResponse = message.decode(&mut self.inner.stream, cnt).await?;
                }
                BackendMessageFormat::DeleteResponse => {
                    let _: DeleteResponse = message.decode(&mut self.inner.stream, cnt).await?;
                }
                BackendMessageFormat::UpdateResponse => {
                    let _: UpdateResponse = message.decode(&mut self.inner.stream, cnt).await?;
                }
                BackendMessageFormat::ParameterDescription => {
                    let _: ParameterDescription =
                        message.decode(&mut self.inner.stream, cnt).await?;
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    async fn handle_ready_for_query(&mut self) -> Result<(), Error> {
        self.inner.pending_ready_for_query_count = self
            .inner
            .pending_ready_for_query_count
            .checked_sub(1)
            .ok_or_else(|| err_protocol!("received more ReadyForQuery messages than expected"))?;

        Ok(())
    }

    pub(crate) fn in_transaction(&self) -> bool {
        // TODO in_transaction
        // self.inner
        //     .status_flags
        //     .intersects(Status::SERVER_STATUS_IN_TRANS)
        true
    }

    /// 发送中断信号,停止接受服务器返回数据
    pub(crate) async fn send_halt(&mut self) -> Result<(), Error> {
        let buf = b".".as_slice();
        self.inner.stream.send_packet(buf).await?;

        Ok(())
    }
}

impl Debug for XuguConnection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XuguConnection").finish()
    }
}

impl Connection for XuguConnection {
    type Database = Xugu;
    type Options = XuguConnectOptions;

    fn close(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            // self.inner.stream.send_packet(Quit).await?;
            // TODO
            self.send_halt().await?;
            self.inner.stream.shutdown().await?;
            Ok(())
        })
    }

    fn close_hard(mut self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            self.inner.stream.shutdown().await?;
            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.wait_until_ready().await?;
            self.inner.stream.send_packet(Ping).await?;
            let _ok: OkPacket = self.inner.stream.recv().await?;

            Ok(())
        })
    }

    fn begin(&mut self) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self, None)
    }

    fn begin_with(
        &mut self,
        statement: impl Into<Cow<'static, str>>,
    ) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self, Some(statement.into()))
    }

    fn cached_statements_size(&self) -> usize {
        self.inner.cache_statement.len()
    }

    fn clear_cached_statements(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            self.wait_until_ready().await?;

            while let Some((statement_id, _)) = self.inner.cache_statement.remove_lru() {
                self.inner
                    .stream
                    .send_packet(StmtClose {
                        st_id: statement_id,
                        con_obj_name: &self.inner.con_obj_name,
                    })
                    .await?;

                let _ok: OkPacket = self.inner.stream.recv().await?;
            }

            Ok(())
        })
    }

    fn shrink_buffers(&mut self) {
        self.inner.stream.shrink_buffers();
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        self.wait_until_ready().boxed()
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.inner.stream.write_buffer().is_empty()
    }
}
