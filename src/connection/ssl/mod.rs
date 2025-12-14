use self::rsa::rsa_encrypt;
use turing_cipher::Turing;

mod rsa;

#[allow(dead_code)]
pub struct TurningComputer {
    turing: Turing,
    pos: usize,
    key_stream: [u8; 340],
}

#[allow(dead_code)]
impl TurningComputer {
    pub fn new() -> TurningComputer {
        TurningComputer {
            turing: Turing::new(),
            pos: 0,
            key_stream: [0; 340],
        }
    }

    pub fn init(&mut self, key: &[u8], iv: &[u8]) {
        self.turing.turing_key(key, key.len()).unwrap();
        self.turing.turing_iv(iv, iv.len()).unwrap();
        self.turing.turing_gen(&mut self.key_stream).unwrap();
        self.pos = 0;
    }

    pub fn xor_buff(&mut self, buf: &mut [u8], size: usize) {
        for i in 0..buf.len().min(size) {
            buf[i] ^= self.key_stream[self.pos % 340];
            self.pos += 1;
            if self.pos == 3400 {
                self.turing.turing_gen(&mut self.key_stream).unwrap();
                self.pos = 0;
            }
        }
    }

    pub fn xor_buff_exact(&mut self, buf: &mut [u8]) {
        self.xor_buff(buf, buf.len());
    }

    pub fn xor_i8(&mut self, num: i8) -> i8 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        i8::from_be_bytes(buf)
    }

    pub fn xor_u8(&mut self, num: u8) -> u8 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        u8::from_be_bytes(buf)
    }

    pub fn xor_i16(&mut self, num: i16) -> i16 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        i16::from_be_bytes(buf)
    }

    pub fn xor_u16(&mut self, num: u16) -> u16 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        u16::from_be_bytes(buf)
    }

    pub fn xor_i32(&mut self, num: i32) -> i32 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        i32::from_be_bytes(buf)
    }

    pub fn xor_u32(&mut self, num: u32) -> u32 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        u32::from_be_bytes(buf)
    }

    pub fn xor_i64(&mut self, num: i64) -> i64 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        i64::from_be_bytes(buf)
    }

    pub fn xor_u64(&mut self, num: u64) -> u64 {
        let mut buf = num.to_be_bytes();
        self.xor_buff_exact(buf.as_mut_slice());
        u64::from_be_bytes(buf)
    }
}

/// 每30字节插入两个0x07字符
fn insert7(data: &[u8]) -> Vec<u8> {
    // 计算完整块数量和总容量
    let full_chunks = data.len() / 30;
    let total_capacity = data.len() + full_chunks * 2;

    let mut result = Vec::with_capacity(total_capacity);
    let mut remaining = data;

    // 处理每个完整块
    while remaining.len() >= 30 {
        let (chunk, rest) = remaining.split_at(30);
        result.extend_from_slice(chunk);
        result.push(0x07);
        result.push(0x07);
        remaining = rest;
    }

    // 添加剩余数据
    result.extend_from_slice(remaining);
    result
}

/// 加密 连接字符串
pub fn encrypt_conn(con_str: &str, key: &[u8; 32], public_key: &[u8], trail_key: &[u8]) -> Vec<u8> {
    let con_b = insert7(con_str.as_bytes());
    let mut data = Vec::with_capacity(key.len() + con_b.len());
    data.extend_from_slice(key);
    data.extend_from_slice(&con_b);

    rsa_encrypt(data.as_slice(), public_key, trail_key)
}

/// 初始化 turning 密钥
#[allow(dead_code)]
pub fn init_key(read: &mut TurningComputer, send: &mut TurningComputer, key: &[u8; 32]) {
    let iv = [0u8; 0];
    read.init(key, &iv);
    send.init(key, &iv);
}
