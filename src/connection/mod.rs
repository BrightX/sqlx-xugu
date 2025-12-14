use self::stream::XuguStream;
use crate::protocol::statement::StmtClose;
use crate::protocol::text::{OkPacket, Ping};
use crate::statement::XuguStatementMetadata;
use crate::{Xugu, XuguConnectOptions};
use futures_core::future::BoxFuture;
use sqlx_core::common::StatementCache;
use sqlx_core::connection::{Connection, LogSettings};
use sqlx_core::transaction::Transaction;
use sqlx_core::Error;
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
            // self.inner.stream.wait_until_ready().await?;
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
            while let Some((statement_id, _)) = self.inner.cache_statement.remove_lru() {
                self.inner
                    .stream
                    .send_packet(StmtClose {
                        st_id: statement_id,
                        con_obj_name: &self.inner.con_obj_name,
                    })
                    .await?;
            }

            Ok(())
        })
    }

    fn shrink_buffers(&mut self) {
        self.inner.stream.shrink_buffers();
    }

    #[doc(hidden)]
    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        // self.inner.stream.wait_until_ready().boxed()
        Box::pin(async {
            self.inner.stream.before_flush();
            self.inner.stream.flush().await?;
            Ok(())
        })
    }

    #[doc(hidden)]
    fn should_flush(&self) -> bool {
        !self.inner.stream.write_buffer().is_empty()
    }
}
