use crate::io::AsyncStreamExt;
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct DataRow;

impl BackendMessage for DataRow {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::DataRow;

    async fn decode_body<S: AsyncStreamExt>(_: &mut S, _: ServerContext) -> Result<Self, Error> {
        Ok(Self)
    }
}
