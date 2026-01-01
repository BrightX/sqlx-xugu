use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::message::{BackendMessage, BackendMessageFormat};
use crate::protocol::statement::ParameterDef;
use crate::protocol::ServerContext;
use sqlx_core::Error;

#[derive(Debug)]
pub struct ParameterDescription {
    pub(crate) params: Vec<ParameterDef>,
}

impl BackendMessage for ParameterDescription {
    const FORMAT: BackendMessageFormat = BackendMessageFormat::ParameterDescription;

    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        cnt: ServerContext,
    ) -> Result<Self, Error> {
        // 读取服务器返回的参数信息
        let num = stream.read_i32().await?;
        let mut params = Vec::with_capacity(num as usize);
        for _ in 0..num {
            let def = ParameterDef::decode_with(stream, cnt).await?;
            params.push(def);
        }
        Ok(Self { params })
    }
}
