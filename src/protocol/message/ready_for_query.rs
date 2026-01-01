use crate::io::AsyncStreamExt;
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct ReadyForQuery;

impl BackendMessage for ReadyForQuery {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::ReadyForQuery;

    async fn decode_body<S: AsyncStreamExt>(_: &mut S, _: ServerContext) -> Result<Self, Error> {
        Ok(Self)
    }
}
