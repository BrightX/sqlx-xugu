use crate::connection::StatementId;
use crate::protocol::encode_command0;
use sqlx_core::io::ProtocolEncode;
use sqlx_core::Error;

pub struct Prepare<'a> {
    pub query: &'a str,
    pub st_id: StatementId,
}

impl ProtocolEncode<'_, ()> for Prepare<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        let sql = format!("Prepare {} as {}", self.st_id, self.query);
        encode_command0(buf, &sql);
        Ok(())
    }
}
