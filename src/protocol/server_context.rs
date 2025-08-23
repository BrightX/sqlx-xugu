#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServerContext {
    server_version: i16,
}

impl ServerContext {
    pub fn new(server_version: i16) -> Self {
        ServerContext { server_version }
    }
}

#[allow(dead_code)]
impl ServerContext {
    /// 当前连接是否使用 301 协议
    pub fn support_301(&self) -> bool {
        self.server_version > 201
    }

    /// 当前连接是否使用 302 协议
    pub fn support_302(&self) -> bool {
        self.server_version >= 302
    }

    /// 当前连接是否使用 401 协议
    pub fn support_401(&self) -> bool {
        self.server_version >= 401
    }
}
