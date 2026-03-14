use super::stream::XuguStream;
use super::{StatementId, XuguConnection, XuguConnectionInner};
use crate::XuguConnectOptions;
use sqlx_core::common::StatementCache;
use sqlx_core::Error;

impl XuguConnection {
    pub(crate) async fn establish(options: &XuguConnectOptions) -> Result<Self, Error> {
        let stream = do_handshake(options).await?;

        let inner = XuguConnectionInner {
            stream,
            transaction_depth: 0,
            next_statement_id: StatementId::NAMED_START,
            cache_statement: StatementCache::new(options.statement_cache_capacity),
            pending_ready_for_query_count: 0,
            last_num_columns: 0,
            log_settings: options.log_settings.clone(),
        };

        Ok(Self {
            inner: Box::new(inner),
        })
    }
}

async fn do_handshake(options: &XuguConnectOptions) -> Result<XuguStream, Error> {
    let mut stream = XuguStream::connect(options).await?;
    let conn_str = options.to_conn_str();
    let opts_version = options.get_version();
    stream.do_handshake(&conn_str, opts_version).await?;

    Ok(stream)
}
