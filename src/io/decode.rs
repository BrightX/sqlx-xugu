use crate::io::AsyncStreamExt;
use sqlx_core::Error;

pub(crate) trait StreamDecode<Context = ()>
where
    Self: Sized,
{
    async fn decode_with<S: AsyncStreamExt>(
        stream: &mut S,
        context: Context,
    ) -> Result<Self, Error>;
}
