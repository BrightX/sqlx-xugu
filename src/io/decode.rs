use sqlx_core::Error;
use crate::io::AsyncStreamExt;

pub(crate) trait StreamDecode<Context = ()>
where
    Self: Sized,
{
    async fn decode_with<S: AsyncStreamExt>(stream: &mut S, context: Context) -> Result<Self, Error>;
}
