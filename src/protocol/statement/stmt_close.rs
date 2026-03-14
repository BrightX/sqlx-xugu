use crate::connection::StatementId;
use crate::protocol::encode_command0;
use sqlx_core::io::ProtocolEncode;
use sqlx_core::Error;

#[derive(Debug)]
pub(crate) struct StmtClose(pub(crate) StatementId);

impl ProtocolEncode<'_, ()> for StmtClose {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        let sql = format!("deallocate {}", self.0);
        encode_command0(buf, &sql);
        Ok(())
    }
}
