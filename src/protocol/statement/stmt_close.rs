use crate::protocol::encode_command0;
use sqlx_core::io::ProtocolEncode;
use sqlx_core::Error;

#[derive(Debug)]
pub struct StmtClose<'c> {
    pub con_obj_name: &'c str,
    pub st_id: u32,
}

impl ProtocolEncode<'_, ()> for StmtClose<'_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        let sql = format!("deallocate st{}{}", self.con_obj_name, self.st_id);
        encode_command0(buf, &sql);
        Ok(())
    }
}
