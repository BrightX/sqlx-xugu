use crate::arguments::XuguArgumentValue;
use crate::protocol::encode_sql_command;
use crate::protocol::statement::ParameterDef;
use crate::protocol::text::ColumnType;
use crate::XuguArguments;
use bytes::BufMut;
use sqlx_core::io::ProtocolEncode;
use sqlx_core::Error;

#[derive(Debug)]
pub struct Execute<'c, 'q, 'p> {
    pub con_obj_name: &'c str,
    pub st_id: u32,
    pub arguments: &'q XuguArguments<'q>,
    pub params: &'p Vec<ParameterDef>,
}

impl Execute<'_, '_, '_> {
    fn encode_params(&self, buf: &mut Vec<u8>) {
        let params = &self.params;
        let args = &self.arguments.values;
        let types = &self.arguments.types;

        let args_count = args.len() as i32;
        buf.put_i32(args_count);

        for i in 0..args.len() {
            let param_name = &params[i].param_name;
            buf.put_i16(param_name.len() as i16);
            buf.put_slice(param_name.as_bytes());
            let inout_type = params[i].ordinal;
            buf.put_i16(inout_type as i16);

            let type_id = types[i].r#type as i32;
            let (arg, type_id) = match &args[i] {
                XuguArgumentValue::Null => ([].as_slice(), ColumnType::NULL as i32),
                XuguArgumentValue::Str(x) => (x.as_bytes(), type_id),
                XuguArgumentValue::Bin(x) => (x.as_ref(), type_id),
            };
            buf.put_i16(type_id as i16);

            buf.put_i32(arg.len() as i32);
            buf.put_slice(arg);
        }
    }
}

impl ProtocolEncode<'_, ()> for Execute<'_, '_, '_> {
    fn encode_with(&self, buf: &mut Vec<u8>, _: ()) -> Result<(), Error> {
        let sql_cmd = format!("? st{}{}", self.con_obj_name, self.st_id);

        encode_sql_command(buf, &sql_cmd);
        self.encode_params(buf);
        Ok(())
    }
}
