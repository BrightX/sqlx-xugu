use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::text::ColumnDefinition;
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct RowDescription {
    pub fields: Vec<ColumnDefinition>,
}

impl BackendMessage for RowDescription {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::RowDescription;

    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        cnt: ServerContext,
    ) -> Result<Self, Error> {
        let num_columns = stream.read_i32().await?;
        let mut fields = Vec::with_capacity(num_columns as usize);

        for _ in 0..num_columns {
            let def = ColumnDefinition::decode_with(stream, cnt).await?;
            fields.push(def);
        }
        Ok(Self { fields })
    }
}
