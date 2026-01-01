use crate::io::AsyncStreamExt;
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct ErrorResponse {
    pub error: String,
}

impl BackendMessage for ErrorResponse {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::ErrorResponse;

    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        _: ServerContext,
    ) -> Result<Self, Error> {
        let err = stream.read_str().await?;

        Ok(Self { error: err })
    }
}
