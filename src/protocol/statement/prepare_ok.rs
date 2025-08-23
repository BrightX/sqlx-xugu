use crate::error::Error;
use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::text::{ColumnDefinition, ColumnType};
use crate::protocol::ServerContext;
use crate::XuguDatabaseError;
use sqlx_core::err_protocol;

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

#[derive(Debug)]
#[allow(unused)]
pub(crate) struct PrepareOk {
    pub(crate) columns: Vec<ColumnDefinition>,
    pub(crate) params: Vec<ParameterDef>,
    pub(crate) warnings: Vec<String>,
}

impl StreamDecode<ServerContext> for PrepareOk {
    async fn decode_with<S: AsyncStreamExt>(
        stream: &mut S,
        cnt: ServerContext,
    ) -> Result<Self, Error> {
        let mut warnings = Vec::new();
        let mut error = None;
        let mut columns = Vec::new();
        let mut params = Vec::new();

        loop {
            let bt = stream.read_u8().await?;
            match bt {
                b'E' | b'F' => {
                    let err = stream.read_str().await?;
                    error = Some(err);
                }
                b'W' | b'M' => {
                    // 读到服务器端返回消息用对话框抛出
                    // 警告和信息
                    let warn = stream.read_str().await?;
                    warnings.push(warn);
                }
                b'K' | b'<' => {
                    //命令结束 / 错误结束
                    break;
                }
                b'A' => {
                    // 接收字段定义
                    let fields_count = stream.read_i32().await?;
                    for _ in 0..fields_count {
                        let def = ColumnDefinition::decode_with(stream, cnt).await?;
                        columns.push(def);
                    }
                }
                b'$' => {
                    // 读取服务器返回的参数信息
                    let num = stream.read_i32().await?;
                    for _ in 0..num {
                        let def = ParameterDef::decode_with(stream, cnt).await?;
                        params.push(def);
                    }
                }
                _ => {
                    return Err(err_protocol!("违反虚谷协议first byte: {}", bt as char));
                }
            }
        }

        if let Some(err) = error {
            return Err(Error::Database(Box::new(XuguDatabaseError::from_str(&err))));
        }

        Ok(Self {
            columns,
            params,
            warnings,
        })
    }
}
