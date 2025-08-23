use crate::connection::XuguConnection;
use crate::Xugu;
use futures_core::future::BoxFuture;
use sqlx_core::executor::Executor;
use sqlx_core::transaction::*;
use sqlx_core::Error;
use std::borrow::Cow;
use crate::protocol::text::Query;

/// Implementation of [`TransactionManager`] for Xugu.
pub struct XuguTransactionManager;

impl TransactionManager for XuguTransactionManager {
    type Database = Xugu;

    fn begin<'conn>(
        conn: &'conn mut XuguConnection,
        statement: Option<Cow<'static, str>>,
    ) -> BoxFuture<'conn, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.inner.transaction_depth;
            let statement = match statement {
                // custom `BEGIN` statements are not allowed if we're already in a transaction
                // (we need to issue a `SAVEPOINT` instead)
                Some(_) if depth > 0 => return Err(Error::InvalidSavePointStatement),
                Some(statement) => statement,
                None => begin_ansi_transaction_sql(depth),
            };
            conn.execute(&*statement).await?;
            if !conn.in_transaction() {
                return Err(Error::BeginFailed);
            }
            conn.inner.transaction_depth += 1;

            Ok(())
        })
    }

    fn commit(conn: &mut XuguConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.inner.transaction_depth;
            if depth > 0 {
                // 虚谷 v11 不支持 事务保存点的释放 RELEASE SAVEPOINT _sqlx_savepoint_1
                // 所以忽略  RELEASE SAVEPOINT 的执行，只执行最后的的 COMMIT
                if depth == 1 { conn.execute(&*commit_ansi_transaction_sql(depth)).await?; }

                conn.inner.transaction_depth = depth - 1;
            }

            Ok(())
        })
    }

    fn rollback(conn: &mut XuguConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let depth = conn.inner.transaction_depth;

            if depth > 0 {
                conn.execute(&*rollback_ansi_transaction_sql(depth)).await?;
                conn.inner.transaction_depth = depth - 1;
            }

            Ok(())
        })
    }

    fn start_rollback(conn: &mut XuguConnection) {
        let depth = conn.inner.transaction_depth;

        if depth > 0 {
            // TODO
            // conn.inner.stream.waiting.push_back(Waiting::Result);
            conn.inner
                .stream
                .write_packet(Query(&rollback_ansi_transaction_sql(depth)))
                .expect("BUG: unexpected error queueing ROLLBACK");

            conn.inner.transaction_depth = depth - 1;
        }
    }

    fn get_transaction_depth(conn: &XuguConnection) -> usize {
        conn.inner.transaction_depth
    }
}
