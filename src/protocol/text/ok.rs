use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::ServerContext;
use crate::XuguDatabaseError;
use sqlx_core::{err_protocol, Error};

/// 该方法执行非 select,delete,insert,update 语句时,返回结果中只有一个字节 'K' ，表示执行成功
#[derive(Debug)]
pub(crate) struct OkPacket;

impl StreamDecode<ServerContext> for OkPacket {
    async fn decode_with<S: AsyncStreamExt>(
        stream: &mut S,
        _: ServerContext,
    ) -> Result<Self, Error> {
        let mut error = None;
        loop {
            let bt = stream.read_u8().await?;
            match bt {
                b'E' | b'F' | b'W' | b'M' => {
                    let err = stream.read_str().await?;
                    error = Some(err);
                }
                b'K' => {
                    //命令结束 / 错误结束
                    break;
                }
                _ => {
                    return Err(err_protocol!("违反虚谷协议first byte: {}", bt as char));
                }
            }
        }

        if let Some(err) = error {
            return Err(Error::Database(Box::new(XuguDatabaseError::from_str(&err))));
        }

        Ok(OkPacket)
    }
}
