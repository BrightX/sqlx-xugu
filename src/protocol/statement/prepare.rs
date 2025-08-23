use sqlx_core::Error;
use sqlx_core::io::ProtocolEncode;
use crate::protocol::encode_command0;

pub struct Prepare<'a, 'c> {
    pub query: &'a str,
    pub con_obj_name: &'c str,
    pub st_id: u32,
}

impl ProtocolEncode<'_, ()> for Prepare<'_, '_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        let sql = format!("Prepare st{}{} as {}", self.con_obj_name, self.st_id, self.query);
        encode_command0(buf, &sql);
        Ok(())
    }
}
