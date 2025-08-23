mod connect;
mod parse;

use sqlx_core::connection::LogSettings;

#[derive(Debug, Clone)]
pub struct XuguConnectOptions {
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) user: String,
    pub(crate) password: String,
    pub(crate) database: String,
    pub(crate) charset: String,
    version: Option<i16>,
    return_schema: bool,
    return_rowid: bool,
    encryptor: Option<String>,
    time_zone: Option<String>,
    iso_level: Option<String>,
    lock_timeout: Option<String>,
    lob_ret: Option<String>,
    identity_mode: Option<String>,
    keyword_filter: Option<String>,
    disable_binlog: Option<String>,
    auto_commit: bool,
    pub(crate) use_ssl: bool,
    current_schema: Option<String>,
    compatible_mode: Option<String>,

    pub(crate) log_settings: LogSettings,
}

impl Default for XuguConnectOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl XuguConnectOptions {
    pub(crate) fn to_conn_str(&self) -> String {
        let return_schema = if self.return_schema { "on" } else { "off" };
        let version = self.get_version();
        // 必要参数
        let mut conn_str = format!(
            "login database='{}' user='{}' password='{}' version={} return_schema={} return_cursor_id=on",
            self.database, self.user, self.password, version, return_schema,
        );

        // 可选参数
        if self.return_rowid {
            conn_str += " return_rowid=true";
        }
        if let Some(encryptor) = &self.encryptor {
            conn_str += " encryptor=";
            conn_str += encryptor;
        }
        if self.charset.is_empty() {
            conn_str += " char_set=utf8";
        } else {
            conn_str = conn_str + " char_set=" + self.charset.as_str();
        }
        if let Some(time_zone) = &self.time_zone {
            conn_str = conn_str + " time_zone='" + time_zone + "'";
        }
        if let Some(iso_level) = &self.iso_level {
            conn_str += " iso_level='";
            conn_str += iso_level;
            conn_str += "'";
        }
        if let Some(lock_timeout) = &self.lock_timeout {
            conn_str += " lock_timeout=";
            conn_str += lock_timeout;
        }
        if let Some(lob_ret) = &self.lob_ret {
            conn_str += " lob_ret='";
            conn_str += lob_ret;
            conn_str += "'";
        }
        if let Some(identity_mode) = &self.identity_mode {
            conn_str += " identity_mode='";
            conn_str += identity_mode;
            conn_str += "'";
        }
        if let Some(keyword_filter) = &self.keyword_filter {
            conn_str += " keyword_filter='";
            conn_str += keyword_filter;
            conn_str += "'";
        }
        if let Some(disable_binlog) = &self.disable_binlog {
            conn_str += " disable_binlog='";
            conn_str += disable_binlog;
            conn_str += "'";
        }
        if self.auto_commit {
            conn_str += " auto_commit=on";
        } else {
            conn_str += " auto_commit=off";
        }
        if let Some(current_schema) = &self.current_schema {
            conn_str += " current_schema='";
            conn_str += current_schema;
            conn_str += "'";
        }
        if let Some(compatible_mode) = &self.compatible_mode {
            conn_str += " compatible_mode='";
            conn_str += compatible_mode;
            conn_str += "'";
        }

        conn_str += "\0";
        conn_str
    }

    pub fn get_version(&self) -> i16 {
        self.version.unwrap_or(301)
    }
}

impl XuguConnectOptions {
    /// Creates a new, default set of options ready for configuration
    pub fn new() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 5138,
            database: String::from("SYSTEM"),
            user: String::from("SYSDBA"),
            password: String::from("SYSDBA"),
            version: None,
            return_schema: true,
            return_rowid: false,
            encryptor: None,
            charset: String::from("utf8"),
            time_zone: None,
            iso_level: Some(String::from("READ COMMITTED")),
            lock_timeout: None,
            lob_ret: None,
            identity_mode: None,
            keyword_filter: None,
            disable_binlog: None,
            auto_commit: true,
            use_ssl: false,
            current_schema: None,
            compatible_mode: None,

            log_settings: Default::default(),
        }
    }

    /// Sets the name of the host to connect to.
    ///
    /// The default behavior when the host is not specified,
    /// is to connect to localhost.
    pub fn host(mut self, host: &str) -> Self {
        host.clone_into(&mut self.host);
        self
    }

    /// Sets the port to connect to at the server host.
    ///
    /// The default port for MySQL is `3306`.
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Sets the database name.
    pub fn database(mut self, database: &str) -> Self {
        self.database = database.to_owned();
        self
    }

    /// Sets the user to connect as.
    pub fn user(mut self, user: &str) -> Self {
        user.clone_into(&mut self.user);
        self
    }

    /// Sets the password to connect with.
    pub fn password(mut self, password: &str) -> Self {
        self.password = password.to_owned();
        self
    }

    /// Sets the version to connect with.
    pub fn version(mut self, version: i16) -> Self {
        self.version = Some(version);
        self
    }

    pub fn return_schema(mut self, value: bool) -> Self {
        self.return_schema = value;
        self
    }

    pub fn return_rowid(mut self, value: bool) -> Self {
        self.return_schema = value;
        self
    }

    pub fn encryptor(mut self, value: &str) -> Self {
        self.encryptor = Some(value.into());
        self
    }

    pub fn iso_level(mut self, value: &str) -> Self {
        self.iso_level = Some(value.into());
        self
    }

    pub fn lock_timeout(mut self, value: &str) -> Self {
        self.lock_timeout = Some(value.into());
        self
    }

    pub fn lob_ret(mut self, value: &str) -> Self {
        self.lob_ret = Some(value.into());
        self
    }

    pub fn identity_mode(mut self, value: &str) -> Self {
        self.identity_mode = Some(value.into());
        self
    }

    pub fn keyword_filter(mut self, value: &str) -> Self {
        self.keyword_filter = Some(value.into());
        self
    }

    pub fn disable_binlog(mut self, value: &str) -> Self {
        self.disable_binlog = Some(value.into());
        self
    }

    pub fn auto_commit(mut self, value: bool) -> Self {
        self.auto_commit = value;
        self
    }

    pub fn use_ssl(mut self, value: bool) -> Self {
        self.use_ssl = value;
        self
    }

    pub fn current_schema(mut self, value: &str) -> Self {
        self.current_schema = Some(value.into());
        self
    }

    pub fn compatible_mode(mut self, value: &str) -> Self {
        self.compatible_mode = Some(value.into());
        self
    }

    /// Sets the character set for the connection.
    ///
    /// The default character set is `utf8mb4`. This is supported from MySQL 5.5.3.
    /// If you need to connect to an older version, we recommend you to change this to `utf8`.
    ///
    /// Implies [`.set_names(true)`][Self::set_names()].
    pub fn charset(mut self, charset: &str) -> Self {
        charset.clone_into(&mut self.charset);
        self
    }

    /// If `Some`, sets the `time_zone` option to the given string after connecting to the database.
    ///
    /// If `None`, no `time_zone` parameter is sent; the server timezone will be used instead.
    ///
    /// Defaults to `Some(String::from("+00:00"))` to ensure all timestamps are in UTC.
    ///
    /// ### Warning
    /// Changing this setting from its default will apply an unexpected skew to any
    /// `time::OffsetDateTime` or `chrono::DateTime<Utc>` value, whether passed as a parameter or
    /// decoded as a result. `TIMESTAMP` values are not encoded with their UTC offset in the MySQL
    /// protocol, so encoding and decoding of these types assumes the server timezone is *always*
    /// UTC.
    ///
    /// If you are changing this option, ensure your application only uses
    /// `time::PrimitiveDateTime` or `chrono::NaiveDateTime` and that it does not assume these
    /// timestamps can be placed on a real timeline without applying the proper offset.
    pub fn timezone(mut self, value: impl Into<Option<String>>) -> Self {
        self.time_zone = value.into();
        self
    }
}

impl XuguConnectOptions {
    /// Get the current host.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_xugu::XuguConnectOptions;
    /// let options = XuguConnectOptions::new()
    ///     .host("127.0.0.1");
    /// assert_eq!(options.get_host(), "127.0.0.1");
    /// ```
    pub fn get_host(&self) -> &str {
        &self.host
    }

    /// Get the server's port.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_xugu::XuguConnectOptions;
    /// let options = XuguConnectOptions::new()
    ///     .port(5138);
    /// assert_eq!(options.get_port(), 5138);
    /// ```
    pub fn get_port(&self) -> u16 {
        self.port
    }

    /// Get the current user.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_xugu::XuguConnectOptions;
    /// let options = XuguConnectOptions::new()
    ///     .user("foo");
    /// assert_eq!(options.get_user(), "foo");
    /// ```
    pub fn get_user(&self) -> &str {
        &self.user
    }

    /// Get the current database name.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_xugu::XuguConnectOptions;
    /// let options = XuguConnectOptions::new()
    ///     .database("SYSTEM");
    /// assert_eq!(options.get_database(), "SYSTEM");
    /// ```
    pub fn get_database(&self) -> &str {
        &self.database
    }

    /// Get the server charset.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use sqlx_xugu::XuguConnectOptions;
    /// let options = XuguConnectOptions::new();
    /// assert_eq!(options.get_charset(), "utf8");
    /// ```
    pub fn get_charset(&self) -> &str {
        &self.charset
    }
}
