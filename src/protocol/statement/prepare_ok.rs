use crate::error::Error;
use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::text::ColumnType;
use crate::protocol::ServerContext;

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct ParameterDef {
    pub(crate) param_name: String,
    pub(crate) ordinal: i32,
    pub(crate) r#type: ColumnType,
    pub(crate) precision: i32,
    pub(crate) scale: i32,
}

impl StreamDecode<ServerContext> for ParameterDef {
    async fn decode_with<S: AsyncStreamExt>(
        stream: &mut S,
        _: ServerContext,
    ) -> Result<Self, Error> {
        let param_name = stream.read_str().await?;
        let ordinal = stream.read_i32().await? + 1;
        let type_id = stream.read_i32().await?;
        let precision_scale = stream.read_i32().await?;

        let r#type = ColumnType::try_from_i32(type_id)?;
        let precision;
        let scale;
        if r#type == ColumnType::NUMERIC {
            precision = precision_scale >> 16;
            scale = precision_scale & 0x0000ffff;
        } else {
            precision = precision_scale;
            scale = 0;
        }

        Ok(Self {
            param_name,
            ordinal,
            r#type,
            precision,
            scale,
        })
    }
}
