use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use bytes::Buf;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use sqlx_core::Error;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

const ERROR: &str = "[E50044]error decoding LINE";

/// ## Xugu Geometric Line type
///
/// Description: Infinite line
/// Representation: `{A, B, C}`
///
/// Lines are represented by the linear equation Ax + By + C = 0, where A and B are not both zero.
///
#[derive(Debug, Clone, PartialEq)]
pub struct XgLine {
    pub a: f64,
    pub b: f64,
    pub c: f64,
}

impl Type<Xugu> for XgLine {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::LINE)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::CHAR | ColumnType::LINE | ColumnType::LINE_OLD
        )
    }
}

impl<'r> Decode<'r, Xugu> for XgLine {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = value.as_str()?;
        Ok(Self::from_str(s)?)
    }
}

impl Encode<'_, Xugu> for XgLine {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let s = self.to_string();

        args.push(XuguArgumentValue::Str(Cow::Owned(s)));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl Display for XgLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{},{},{}}}", self.a, self.b, self.c)
    }
}

impl FromStr for XgLine {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s
            .trim_matches(|c| c == '{' || c == '}' || c == ' ')
            .split(',');

        let a = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get a from {}", ERROR, s))?;

        let b = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get b from {}", ERROR, s))?;

        let c = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get c from {}", ERROR, s))?;

        if parts.next().is_some() {
            return Err(format!("{}: too many numbers inputted in {}", ERROR, s).into());
        }

        Ok(Self { a, b, c })
    }
}

impl XgLine {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, Error> {
        let a = bytes.get_f64();
        let b = bytes.get_f64();
        let c = bytes.get_f64();
        Ok(Self { a, b, c })
    }
}
