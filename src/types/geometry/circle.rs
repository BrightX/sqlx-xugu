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

const ERROR: &str = "[E50044]error decoding CIRCLE";

/// ## Xugu Geometric Circle type
///
/// Description: Circle
/// Representation: `< (x, y), radius >` (center point and radius)
///
/// ```text
/// < ( x , y ) , radius >
/// ( ( x , y ) , radius )
///   ( x , y ) , radius
///     x , y   , radius
/// ```
/// where `(x,y)` is the center point.
///
#[derive(Debug, Clone, PartialEq)]
pub struct XgCircle {
    pub x: f64,
    pub y: f64,
    pub radius: f64,
}

impl Type<Xugu> for XgCircle {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::CIRCLE)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(ty.r#type, ColumnType::CHAR | ColumnType::CIRCLE)
    }
}

impl<'r> Decode<'r, Xugu> for XgCircle {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = value.as_str()?;
        Ok(Self::from_str(s)?)
    }
}

impl Encode<'_, Xugu> for XgCircle {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let s = self.to_string();

        args.push(XuguArgumentValue::Str(Cow::Owned(s)));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl Display for XgCircle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<({},{}),{}>", self.x, self.y, self.radius)
    }
}

impl FromStr for XgCircle {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitised = s.replace(['<', '>', '(', ')', ' '], "");
        let mut parts = sanitised.split(',');

        let x = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get x from {}", ERROR, s))?;

        let y = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get y from {}", ERROR, s))?;

        let radius = parts
            .next()
            .and_then(|s| s.trim().parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get radius from {}", ERROR, s))?;

        if parts.next().is_some() {
            return Err(format!("{}: too many numbers inputted in {}", ERROR, s).into());
        }

        if radius < 0. {
            return Err(format!("{}: cannot have negative radius: {}", ERROR, s).into());
        }

        Ok(XgCircle { x, y, radius })
    }
}

impl XgCircle {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<XgCircle, Error> {
        let x = bytes.get_f64();
        let y = bytes.get_f64();
        let r = bytes.get_f64();
        Ok(XgCircle { x, y, radius: r })
    }
}
