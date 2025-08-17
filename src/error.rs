use std::error::Error as StdError;
use std::fmt::{self, Debug, Display, Formatter};

pub(crate) use sqlx_core::error::*;
use std::borrow::Cow;
use std::ops::Range;

pub struct XuguDatabaseError {
    code: String,
    message: String,
}

impl Debug for XuguDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("XuguDatabaseError")
            .field("code", &self.code)
            .field("message", &self.message)
            .finish()
    }
}

impl Display for XuguDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(code) = &self.code() {
            write!(f, "({}): {}", code, self.message())
        } else {
            write!(f, "{}", self.message())
        }
    }
}

impl XuguDatabaseError {
    pub fn from_str(err: &str) -> Self {
        let mut code_range: Range<usize> = 0..0;
        let mut message_range = 0..err.len();
        if err.starts_with("[E") {
            if let Some(pos) = err.find(']') {
                code_range = 1..pos;
                message_range = pos + 1..err.len();
            }
        }

        XuguDatabaseError {
            code: err[code_range].into(),
            message: err[message_range].trim().into(),
        }
    }

    fn get_err_code(code: &str) -> i32 {
        // 去除可能的方括号
        let cleaned = code.trim_matches(|c| c == '[' || c == ']');

        // 查找 E 的位置
        if let Some(e_pos) = cleaned.find('E') {
            let after_e = &cleaned[e_pos + 1..];

            // 检查是否有 L
            if let Some(l_pos) = after_e.find('L') {
                // 有 L 的情况：提取 E 和 L 之间的数字
                let numeric_part = &after_e[..l_pos];
                numeric_part.parse().unwrap_or(0)
            } else {
                // 没有 L 的情况：提取 E 之后的所有数字
                after_e.parse().unwrap_or(0)
            }
        } else {
            // 没有找到 E，尝试直接解析整个字符串为数字
            cleaned.parse().unwrap_or(0)
        }
    }
}

impl StdError for XuguDatabaseError {}

impl DatabaseError for XuguDatabaseError {
    #[inline]
    fn message(&self) -> &str {
        self.message.as_str()
    }

    #[inline]
    fn code(&self) -> Option<Cow<'_, str>> {
        Some(Cow::from(&self.code))
    }

    #[doc(hidden)]
    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    #[doc(hidden)]
    fn into_error(self: Box<Self>) -> Box<dyn StdError + Send + Sync + 'static> {
        self
    }

    fn kind(&self) -> ErrorKind {
        let err_code = Self::get_err_code(&self.code);
        match err_code {
            // E13001：违反唯一值约束
            13001 => ErrorKind::UniqueViolation,
            // E13005：违反外键约束
            13005 => ErrorKind::ForeignKeyViolation,
            // E13009：非空约束作用字段为主键或唯一值字段,故不允许删除
            13009 => ErrorKind::NotNullViolation,
            // E13004：违反约束
            // E13008：违反值检查约束
            13004 | 13008 => ErrorKind::CheckViolation,
            _ => ErrorKind::Other,
        }
    }
}
