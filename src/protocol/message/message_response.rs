use crate::io::AsyncStreamExt;
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct MessageResponse {
    pub msg: String,
}

impl BackendMessage for MessageResponse {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::MessageResponse;

    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        _: ServerContext,
    ) -> Result<Self, Error> {
        let msg = stream.read_str().await?;

        Ok(Self { msg })
    }
}
