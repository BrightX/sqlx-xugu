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

/// ## Xugu Geometric Point type
///
/// Description: Point on a plane
/// Representation: `(x, y)`
///
/// Points are the fundamental two-dimensional building block for geometric types. Values of type point are specified using either of the following syntaxes:
/// ```text
/// ( x , y )
///  x , y
/// ````
/// where x and y are the respective coordinates, as floating-point numbers.
///
#[derive(Debug, Clone, PartialEq)]
pub struct XgPoint {
    pub x: f64,
    pub y: f64,
}

impl Type<Xugu> for XgPoint {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::POINT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::CHAR | ColumnType::POINT | ColumnType::POINT_OLD
        )
    }
}

impl Encode<'_, Xugu> for XgPoint {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let s = self.to_string();

        args.push(XuguArgumentValue::Str(Cow::Owned(s)));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl<'r> Decode<'r, Xugu> for XgPoint {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        if ty == ColumnType::POINT_OLD {
            let buf = value.as_bytes()?;
            return Ok(Self::from_bytes(buf)?);
        }
        // 部分旧版的是按字节解码，新版的是字符串
        if ty == ColumnType::POINT {
            let buf = value.as_bytes()?;
            if buf.len() == 16 && buf[0] != b'(' && !buf.contains(&b',') {
                return Ok(Self::from_bytes(buf)?);
            }
        }

        let s = value.as_str()?;
        Ok(Self::from_str(s)?)
    }
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.trim()
        .parse()
        .map_err(|_| Error::Decode(error_msg.into()))
}

impl Display for XgPoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl FromStr for XgPoint {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (x_str, y_str) = s
            .trim_matches(|c| c == '(' || c == ')' || c == ' ')
            .split_once(',')
            .ok_or_else(|| {
                format!(
                    "[E50044]error decoding POINT: could not get x and y from {}",
                    s
                )
            })?;

        let x = parse_float_from_str(x_str, "[E50044]error decoding POINT: could not get x")?;
        let y = parse_float_from_str(y_str, "[E50044]error decoding POINT: could not get y")?;

        Ok(Self { x, y })
    }
}

impl XgPoint {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, BoxDynError> {
        let x = bytes.get_f64();
        let y = bytes.get_f64();
        Ok(Self { x, y })
    }
}
