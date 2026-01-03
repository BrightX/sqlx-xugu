use bytes::Bytes;
use sqlx_core::Error;

type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub trait AsyncStreamExt {
    async fn read_u8(&mut self) -> Result<u8>;
    async fn read_u16(&mut self) -> Result<u16>;
    async fn read_i32(&mut self) -> Result<i32>;
    async fn read_i64(&mut self) -> Result<i64>;
    async fn read_bytes(&mut self, len: usize) -> Result<Bytes>;
    async fn read_str(&mut self) -> Result<String>;
}
