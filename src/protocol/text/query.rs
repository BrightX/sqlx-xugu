use sqlx_core::Error;
use sqlx_core::io::ProtocolEncode;
use crate::protocol::encode_command0;

#[derive(Debug)]
pub(crate) struct Query<'q>(pub(crate) &'q str);

impl ProtocolEncode<'_, ()> for Query<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        encode_command0(buf, self.0);
        Ok(())
    }
}
