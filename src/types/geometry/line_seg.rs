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

const ERROR: &str = "[E50044]error decoding LSEG";

/// ## Xugu Geometric Line Segment type
///
/// Description: Finite line segment
/// Representation: `((start_x,start_y),(end_x,end_y))`
///
///
/// Line segments are represented by pairs of points that are the endpoints of the segment. Values of type lseg are specified using any of the following syntaxes:
/// ```text
/// [ ( start_x , start_y ) , ( end_x , end_y ) ]
/// ( ( start_x , start_y ) , ( end_x , end_y ) )
///   ( start_x , start_y ) , ( end_x , end_y )
///     start_x , start_y   ,   end_x , end_y
/// ```
/// where `(start_x,start_y) and (end_x,end_y)` are the end points of the line segment.
///
#[doc(alias = "line segment")]
#[derive(Debug, Clone, PartialEq)]
pub struct XgLSeg {
    pub start_x: f64,
    pub start_y: f64,
    pub end_x: f64,
    pub end_y: f64,
}

impl Type<Xugu> for XgLSeg {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::LSEG)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(ty.r#type, ColumnType::CHAR | ColumnType::LSEG)
    }
}

impl<'r> Decode<'r, Xugu> for XgLSeg {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = value.as_str()?;
        Ok(Self::from_str(s)?)
    }
}

impl Encode<'_, Xugu> for XgLSeg {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let s = self.to_string();

        args.push(XuguArgumentValue::Str(Cow::Owned(s)));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl Display for XgLSeg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[({},{}),({},{})]",
            self.start_x, self.start_y, self.end_x, self.end_y
        )
    }
}

impl FromStr for XgLSeg {
    type Err = BoxDynError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitised = s.replace(['(', ')', '[', ']', ' '], "");
        let mut parts = sanitised.split(',');

        let start_x = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get start_x from {}", ERROR, s))?;

        let start_y = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get start_y from {}", ERROR, s))?;

        let end_x = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get end_x from {}", ERROR, s))?;

        let end_y = parts
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| format!("{}: could not get end_y from {}", ERROR, s))?;

        if parts.next().is_some() {
            return Err(format!("{}: too many numbers inputted in {}", ERROR, s).into());
        }

        Ok(Self {
            start_x,
            start_y,
            end_x,
            end_y,
        })
    }
}

impl XgLSeg {
    pub fn from_bytes(mut bytes: &[u8]) -> Result<Self, Error> {
        let start_x = bytes.get_f64();
        let start_y = bytes.get_f64();
        let end_x = bytes.get_f64();
        let end_y = bytes.get_f64();
        Ok(Self {
            start_x,
            start_y,
            end_x,
            end_y,
        })
    }
}
