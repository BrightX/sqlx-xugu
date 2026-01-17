use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use bytes::Buf;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

const ERROR: &str = "[E50044]Error decoding BOX";

/// ## Xugu Geometric Box type
///
/// Description: Rectangular box
/// Representation: `((upper_right_x,upper_right_y),(lower_left_x,lower_left_y))`
///
/// Boxes are represented by pairs of points that are opposite corners of the box. Values of type box are specified using any of the following syntaxes:
///
/// ```text
/// ( ( upper_right_x , upper_right_y ) , ( lower_left_x , lower_left_y ) )
/// ( upper_right_x , upper_right_y ) , ( lower_left_x , lower_left_y )
///   upper_right_x , upper_right_y   ,   lower_left_x , lower_left_y
/// ```
/// where `(upper_right_x,upper_right_y) and (lower_left_x,lower_left_y)` are any two opposite corners of the box.
/// Any two opposite corners can be supplied on input, but the values will be reordered as needed to store the upper right and lower left corners, in that order.
///
#[derive(Debug, Clone, PartialEq)]
pub struct XgBox {
    pub upper_right_x: f64,
    pub upper_right_y: f64,
    pub lower_left_x: f64,
    pub lower_left_y: f64,
}

impl Type<Xugu> for XgBox {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::BOX)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::CHAR | ColumnType::BOX | ColumnType::BOX_OLD | ColumnType::BOX2D
        )
    }
}

impl<'r> Decode<'r, Xugu> for XgBox {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        if ty == ColumnType::BOX_OLD {
            let buf = value.as_bytes()?;
            return Ok(Self::from_bytes(buf)?);
        }
        // 部分旧版的是按字节解码，新版的是字符串
        if ty == ColumnType::BOX {
            let buf = value.as_bytes()?;
            if buf.len() == 32 && buf[0] != b'(' && !buf.contains(&b',') {
                return Ok(Self::from_bytes(buf)?);
            }
        }

        let s = value.as_str()?;
        Ok(Self::from_str(s)?)
    }
}

impl Encode<'_, Xugu> for XgBox {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let s = self.to_string();

        args.push(XuguArgumentValue::Str(Cow::Owned(s)));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl Display for XgBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({},{}),({},{})",
            self.upper_right_x, self.upper_right_y, self.lower_left_x, self.lower_left_y
        )
    }
}

impl FromStr for XgBox {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitised = s.replace(['(', ')', '[', ']', ' '], "");
        let mut parts = sanitised.split(',');

        let upper_right_x = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get upper_right_x from {}", ERROR, s))?;

        let upper_right_y = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get upper_right_y from {}", ERROR, s))?;

        let lower_left_x = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get lower_left_x from {}", ERROR, s))?;

        let lower_left_y = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get lower_left_y from {}", ERROR, s))?;

        if parts.next().is_some() {
            return Err(format!("{}: too many numbers inputted in {}", ERROR, s).into());
        }

        Ok(Self {
            upper_right_x,
            upper_right_y,
            lower_left_x,
            lower_left_y,
        })
    }
}

impl XgBox {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, BoxDynError> {
        let upper_right_x = bytes.get_f64();
        let upper_right_y = bytes.get_f64();
        let lower_left_x = bytes.get_f64();
        let lower_left_y = bytes.get_f64();

        Ok(Self {
            upper_right_x,
            upper_right_y,
            lower_left_x,
            lower_left_y,
        })
    }
}
