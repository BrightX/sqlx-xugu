use crate::connection::XuguConnection;
use crate::XuguConnectOptions;
use futures_core::future::BoxFuture;
use log::LevelFilter;
use sqlx_core::connection::ConnectOptions;
use sqlx_core::{Error, Url};
use std::time::Duration;

impl ConnectOptions for XuguConnectOptions {
    type Connection = XuguConnection;

    fn from_url(url: &Url) -> Result<Self, Error> {
        Self::parse_from_url(url)
    }

    fn to_url_lossy(&self) -> Url {
        self.build_url()
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async {
            let conn = XuguConnection::establish(self).await?;
            Ok(conn)
        })
    }

    fn log_statements(mut self, level: LevelFilter) -> Self {
        self.log_settings.log_statements(level);
        self
    }

    fn log_slow_statements(mut self, level: LevelFilter, duration: Duration) -> Self {
        self.log_settings.log_slow_statements(level, duration);
        self
    }
}
