use super::stream::XuguStream;
use super::{XuguConnection, XuguConnectionInner};
use crate::XuguConnectOptions;
use sqlx_core::common::StatementCache;
use sqlx_core::Error;

impl XuguConnection {
    pub(crate) async fn establish(options: &XuguConnectOptions) -> Result<Self, Error> {
        let stream = do_handshake(options).await?;

        let mut inner = XuguConnectionInner {
            stream,
            transaction_depth: 0,
            cache_statement: StatementCache::new(1024),
            log_settings: options.log_settings.clone(),
            st_id_gen: 0,
            con_obj_name: String::new(),
        };

        let inner_hex = format!("{:02x}", inner.addr_code());
        inner.con_obj_name.push_str(&inner_hex);

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
