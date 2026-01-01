use crate::io::AsyncStreamExt;
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct DeleteResponse {
    pub rows_affected: i32,
}

impl BackendMessage for DeleteResponse {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::DeleteResponse;

    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        _: ServerContext,
    ) -> Result<Self, Error> {
        let rows_affected = stream.read_i32().await?;

        Ok(Self { rows_affected })
    }
}
