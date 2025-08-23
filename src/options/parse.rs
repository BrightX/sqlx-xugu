use crate::XuguConnectOptions;
use sqlx_core::percent_encoding::{percent_decode_str, utf8_percent_encode, NON_ALPHANUMERIC};
use sqlx_core::{Error, Url};
use std::str::FromStr;

fn parse_bool(s: &str, default: bool) -> bool {
    match s {
        "true" | "on" | "1" | "t" | "T" => true,
        "false" | "off" | "0" | "f" | "F" => false,
        _ => default,
    }
}

fn bool2url(b: bool) -> &'static str {
    if b {
        "on"
    } else {
        "off"
    }
}

impl XuguConnectOptions {
    pub(crate) fn parse_from_url(url: &Url) -> Result<Self, Error> {
        let mut options = Self::new();

        if let Some(host) = url.host_str() {
            options = options.host(host);
        }

        if let Some(port) = url.port() {
            options = options.port(port);
        }

        let user = url.username();
        if !user.is_empty() {
            options = options.user(
                &percent_decode_str(user)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        if let Some(password) = url.password() {
            options = options.password(
                &percent_decode_str(password)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        let path = url.path().trim_start_matches('/');
        if !path.is_empty() {
            options = options.database(
                &percent_decode_str(path)
                    .decode_utf8()
                    .map_err(Error::config)?,
            );
        }

        for (key, value) in url.query_pairs().into_iter() {
            if value.is_empty() {
                continue;
            }
            match &*key {
                "user" => options = options.user(&value),
                "password" => options = options.password(&value),
                "version" => options = options.version(value.parse().unwrap()),
                "return_schema" => options = options.return_schema(parse_bool(&value, true)),
                "return_rowid" => options = options.return_rowid(parse_bool(&value, true)),
                "encryptor" => options = options.encryptor(&value),
                "charset" => options = options.charset(&value),
                "timezone" | "time_zone" | "time-zone" => {
                    options = options.timezone(Some(value.replace("GMT ", "GMT+")));
                }
                "iso_level" => options = options.iso_level(&value),
                "lock_timeout" => options = options.lock_timeout(&value),
                "lob_ret" => options = options.lob_ret(&value),
                "identity_mode" => options = options.identity_mode(&value),
                "keyword_filter" => options = options.keyword_filter(&value),
                "disable_binlog" => options = options.disable_binlog(&value),
                "auto_commit" => options = options.auto_commit(parse_bool(&value, true)),
                "current_schema" | "schemaon" => options = options.current_schema(&value),
                "compatible_mode" => options = options.compatible_mode(&value),
                "useSSL" | "usessl" | "use_ssl" => {
                    options = options.use_ssl(parse_bool(&value, false))
                }
                "ssl" if value.eq("ssl") => options = options.use_ssl(true),

                _ => {}
            }
        }

        Ok(options)
    }

    pub(crate) fn build_url(&self) -> Url {
        let mut url = Url::parse(&format!("xugu://{}@{}:{}", self.user, self.host, self.port))
            .expect("BUG: generated un-parseable URL");

        let password = utf8_percent_encode(&self.password, NON_ALPHANUMERIC).to_string();
        let _ = url.set_password(Some(&password));

        url.set_path(&self.database);

        if let Some(version) = self.version {
            url.query_pairs_mut()
                .append_pair("version", version.to_string().as_str());
        }

        url.query_pairs_mut()
            .append_pair("return_schema", bool2url(self.return_schema));
        url.query_pairs_mut()
            .append_pair("return_rowid", bool2url(self.return_rowid));
        if let Some(encryptor) = &self.encryptor {
            url.query_pairs_mut().append_pair("encryptor", &encryptor);
        }
        url.query_pairs_mut().append_pair("charset", &self.charset);
        if let Some(time_zone) = &self.time_zone {
            url.query_pairs_mut().append_pair("time_zone", &time_zone);
        }
        if let Some(iso_level) = &self.iso_level {
            url.query_pairs_mut().append_pair("iso_level", &iso_level);
        }
        if let Some(lock_timeout) = &self.lock_timeout {
            url.query_pairs_mut()
                .append_pair("lock_timeout", &lock_timeout);
        }
        if let Some(lob_ret) = &self.lob_ret {
            url.query_pairs_mut().append_pair("lob_ret", &lob_ret);
        }
        if let Some(identity_mode) = &self.identity_mode {
            url.query_pairs_mut()
                .append_pair("identity_mode", &identity_mode);
        }
        if let Some(keyword_filter) = &self.keyword_filter {
            url.query_pairs_mut()
                .append_pair("keyword_filter", &keyword_filter);
        }
        if let Some(disable_binlog) = &self.disable_binlog {
            url.query_pairs_mut()
                .append_pair("disable_binlog", &disable_binlog);
        }
        url.query_pairs_mut()
            .append_pair("auto_commit", bool2url(self.auto_commit));
        if let Some(current_schema) = &self.current_schema {
            url.query_pairs_mut()
                .append_pair("current_schema", &current_schema);
        }
        if let Some(compatible_mode) = &self.compatible_mode {
            url.query_pairs_mut()
                .append_pair("compatible_mode", &compatible_mode);
        }
        url.query_pairs_mut()
            .append_pair("use_ssl", bool2url(self.use_ssl));

        url
    }
}

impl FromStr for XuguConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Error> {
        let url: Url = s.parse().map_err(Error::config)?;
        Self::parse_from_url(&url)
    }
}
