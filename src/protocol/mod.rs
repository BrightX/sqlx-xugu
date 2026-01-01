pub(crate) mod message;
mod server_context;
pub(crate) mod statement;
pub(crate) mod text;

use bytes::BufMut;

pub(crate) use server_context::ServerContext;

fn encode_sql_command(buf: &mut Vec<u8>, sql_cmd: &str) {
    const FIRST: u8 = b'?';
    const LAST: u8 = b'\0';
    let sql_len = sql_cmd.len();

    buf.push(FIRST);
    buf.put_u32(sql_len as u32);
    buf.extend(sql_cmd.as_bytes());
    buf.push(LAST);
}

fn encode_command0(buf: &mut Vec<u8>, sql_cmd: &str) {
    encode_sql_command(buf, sql_cmd);
    // without params
    buf.put_u32(0);
}
