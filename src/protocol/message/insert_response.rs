use crate::io::AsyncStreamExt;
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct InsertResponse {
    pub rowid: String,
}

impl BackendMessage for InsertResponse {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::InsertResponse;

    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        cnt: ServerContext,
    ) -> Result<Self, Error> {
        let rowid = stream.read_str().await?;
        if cnt.support_302() {
            let col_no = stream.read_i32().await?;
            if col_no >= 0 {
                let identity = stream.read_bytes(8).await?;
                // TODO v302 rowid
                println!("identity: {:?}", identity);
            }
        }

        Ok(Self { rowid })
    }
}
