use super::ssl::{encrypt_conn, init_key, TurningComputer};
use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::ServerContext;
use crate::{XuguConnectOptions, XuguDatabaseError};
use sqlx_core::bytes::{Buf, Bytes};
use sqlx_core::io::ProtocolEncode;
use sqlx_core::net::{connect_tcp, BufferedSocket, Socket, SocketIntoBox};
use sqlx_core::{err_protocol, Error};
use std::ops::{ControlFlow, Deref, DerefMut};

type Result<T> = std::result::Result<T, Error>;

pub struct XuguStream {
    socket: BufferedSocket<Box<dyn Socket>>,
    turing_read: TurningComputer,
    turing_send: TurningComputer,
    use_ssl: bool,
    /// 服务器协议版本号（201老协议，301新协议）
    pub(crate) server_version: i16,
}

impl AsyncStreamExt for XuguStream {
    async fn read_u8(&mut self) -> Result<u8> {
        let mut num = self.read_bytes(1).await?;
        Ok(num.get_u8())
    }

    async fn read_u16(&mut self) -> Result<u16> {
        let mut num = self.read_bytes(2).await?;
        Ok(num.get_u16())
    }

    async fn read_i32(&mut self) -> Result<i32> {
        let mut num = self.read_bytes(4).await?;
        Ok(num.get_i32())
    }

    async fn read_i64(&mut self) -> Result<i64> {
        let mut num = self.read_bytes(8).await?;
        Ok(num.get_i64())
    }

    async fn read_bytes(&mut self, len: usize) -> Result<Bytes> {
        let mut buf = self.socket.read_buffered(len).await?;
        if self.use_ssl {
            self.turing_read.xor_buff_exact(buf.as_mut());
        }
        Ok(buf.freeze())
    }

    async fn read_str(&mut self) -> Result<String> {
        let len = self.read_i32().await?;
        if len <= 0 {
            return Ok(String::new());
        }
        let bytes = self.read_bytes(len as usize).await?;

        Ok(String::from_utf8_lossy(trim_nul_end(&bytes)).into_owned())
    }
}

impl XuguStream {
    /// 读取缓冲区已到达的剩余数据
    async fn read_buf(&mut self) -> Result<Bytes> {
        self.socket
            .try_read(|buf| Ok(ControlFlow::Break(buf.split_to(buf.len()))))
            .await
            .map(|mut buf| {
                if self.use_ssl {
                    self.turing_read.xor_buff_exact(buf.as_mut());
                }
                buf.freeze()
            })
    }
}

fn trim_nul_end(mut bytes: &[u8]) -> &[u8] {
    // Note: A pattern matching based approach (instead of indexing) allows
    // making the function const.
    while let [rest @ .., last] = bytes {
        if *last == b'\0' {
            bytes = rest;
        } else {
            break;
        }
    }
    bytes
}

impl XuguStream {
    pub(super) async fn connect(options: &XuguConnectOptions) -> Result<Self> {
        let host = options.host.as_str();
        let port = options.port;
        let socket = connect_tcp(host, port, SocketIntoBox).await?;

        Ok(Self {
            socket: BufferedSocket::new(socket),
            turing_read: TurningComputer::new(),
            turing_send: TurningComputer::new(),
            use_ssl: options.use_ssl,
            server_version: 201,
        })
    }

    pub(super) async fn do_handshake(&mut self, conn_str: &str, opts_version: i16) -> Result<bool> {
        if !self.use_ssl {
            self.handshake(conn_str).await?;
        } else {
            self.handshake_ssl(conn_str).await?;
        };

        self.handshake_recv(opts_version).await
    }

    pub(crate) fn before_flush(&mut self) {
        if self.use_ssl {
            let buf = self.socket.write_buffer_mut().get_mut();
            self.turing_send.xor_buff_exact(buf);
        }
    }

    async fn handshake(&mut self, conn_str: &str) -> Result<()> {
        self.socket.write(conn_str.as_bytes())?;
        self.socket.flush().await?;
        Ok(())
    }

    async fn handshake_ssl(&mut self, conn_str: &str) -> Result<()> {
        self.socket.write(b"~ssl~".as_slice())?;
        self.socket.flush().await?;
        let mut public_key: Bytes = self.socket.read(32).await?;
        let mut trail_key: Bytes = self.socket.read(32).await?;

        let mut key = [0u8; 32];
        getrandom::fill(&mut key).unwrap();
        // 正大整数 符号位为 0
        key[31] &= 0b0111_1111;

        init_key(&mut self.turing_read, &mut self.turing_send, &key);
        let conn_bytes = encrypt_conn(&conn_str, &key, &mut public_key, &mut trail_key);
        self.socket.write(conn_bytes.as_slice())?;
        self.socket.flush().await?;

        Ok(())
    }

    async fn handshake_recv(&mut self, opts_version: i16) -> Result<bool> {
        let cmd = self.read_u8().await?;
        match cmd {
            b'K' | b'N' => {
                if cmd == b'N' {
                    // 消耗剩余参数
                    let _n = self.read_buf().await?;
                    let _v = _n.to_vec();
                    self.server_version = opts_version;
                }
                return Ok(true);
            }
            b'E' | b'F' => {
                let err_msg = self.read_str().await?;
                return Err(Error::Database(Box::new(XuguDatabaseError::from_str(
                    &err_msg,
                ))));
            }
            _ => (),
        }

        Err(err_protocol!("ssl 握手 cmd: {} -> {}", cmd, cmd as char))
    }
}

impl XuguStream {
    pub(crate) async fn send_packet<'en, T>(&mut self, payload: T) -> Result<()>
    where
        T: ProtocolEncode<'en, ()>,
    {
        self.write_packet(payload)?;
        self.before_flush();
        self.socket.flush().await?;
        Ok(())
    }

    pub(crate) fn write_packet<'en, T>(&mut self, payload: T) -> Result<()>
    where
        T: ProtocolEncode<'en, ()>,
    {
        self.socket.write_with(payload, ())
    }

    pub(crate) async fn recv<T>(&mut self) -> Result<T>
    where
        T: StreamDecode<ServerContext>,
    {
        T::decode_with(self, ServerContext::new(self.server_version)).await
    }
}

impl Deref for XuguStream {
    type Target = BufferedSocket<Box<dyn Socket>>;

    fn deref(&self) -> &Self::Target {
        &self.socket
    }
}

impl DerefMut for XuguStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.socket
    }
}
