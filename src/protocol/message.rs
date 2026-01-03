mod data_row;
mod delete_response;
mod error_response;
mod insert_response;
mod message_response;
mod parameter_description;
mod ready_for_query;
mod row_description;
mod update_response;

use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::ServerContext;
use futures_util::TryFutureExt;
use sqlx_core::{err_protocol, Error};

pub(crate) use data_row::DataRow;
pub(crate) use delete_response::DeleteResponse;
pub(crate) use error_response::ErrorResponse;
pub(crate) use insert_response::InsertResponse;
pub(crate) use message_response::MessageResponse;
pub(crate) use parameter_description::ParameterDescription;
pub(crate) use ready_for_query::ReadyForQuery;
pub(crate) use row_description::RowDescription;
pub(crate) use update_response::UpdateResponse;

#[derive(Debug, PartialOrd, PartialEq)]
#[repr(u8)]
pub enum BackendMessageFormat {
    ErrorResponse,
    MessageResponse,
    ReadyForQuery,
    InsertResponse,
    DeleteResponse,
    UpdateResponse,
    // 字段定义
    RowDescription,
    // 参数定义
    ParameterDescription,
    // 行数据
    DataRow,
}

impl BackendMessageFormat {
    pub fn try_from_u8(v: u8) -> Result<Self, Error> {
        let t = match v {
            // 异常消息字符串
            b'E' | b'F' => Self::ErrorResponse,
            // 服务器端返回消息、警告和信息
            b'W' | b'M' => Self::MessageResponse,
            // 命令结束 / 错误结束
            b'K' | b'<' => Self::ReadyForQuery,
            // 插入 返回插入的 rowid
            b'I' => Self::InsertResponse,
            // 删除影响行数
            b'D' => Self::DeleteResponse,
            // 更新影响行数
            b'U' => Self::UpdateResponse,
            // 接收字段定义
            b'A' => Self::RowDescription,
            // 读取服务器返回的参数信息
            b'$' => Self::ParameterDescription,
            // 接收行数据
            b'R' => Self::DataRow,
            b'S' => return Err(err_protocol!("未实现 虚谷协议first byte: {}", v as char)),
            b'L' => return Err(err_protocol!("未实现 虚谷协议first byte: {}", v as char)),
            b'P' => return Err(err_protocol!("未实现 虚谷协议first byte: {}", v as char)),
            b'O' => return Err(err_protocol!("未实现 虚谷协议first byte: {}", v as char)),

            _ => return Err(err_protocol!("违反虚谷协议first byte: {}", v as char)),
        };

        Ok(t)
    }
}

#[derive(Debug)]
pub struct ReceivedMessage {
    pub format: BackendMessageFormat,
}

impl ReceivedMessage {
    #[inline]
    pub async fn decode<T, S>(self, stream: &mut S, cnt: ServerContext) -> Result<T, Error>
    where
        T: BackendMessage,
        S: AsyncStreamExt,
    {
        if T::FORMAT != self.format {
            return Err(err_protocol!(
                "Xugu protocol error: expected {:?}, got {:?}",
                T::FORMAT,
                self.format
            ));
        }

        Ok(T::decode_body(stream, cnt)
            .map_err(|e| match e {
                Error::Protocol(s) => {
                    err_protocol!("Xugu protocol error (reading {:?}): {s}", self.format)
                }
                other => other,
            })
            .await?)
    }
}

impl StreamDecode<ServerContext> for ReceivedMessage {
    async fn decode_with<S: AsyncStreamExt>(
        stream: &mut S,
        _: ServerContext,
    ) -> Result<Self, Error> {
        let bt = stream.read_u8().await?;
        let format = BackendMessageFormat::try_from_u8(bt)?;

        Ok(ReceivedMessage { format })
    }
}

pub(crate) trait BackendMessage: Sized {
    /// The expected message format.
    const FORMAT: BackendMessageFormat;

    /// Decode this type from a Backend message in the protocol.
    ///
    /// The format code and length prefix have already been read and are not at the start of `bytes`.
    async fn decode_body<S: AsyncStreamExt>(
        stream: &mut S,
        cnt: ServerContext,
    ) -> Result<Self, Error>;
}
