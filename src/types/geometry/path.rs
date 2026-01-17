use super::XgPoint;
use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use sqlx_core::Error;
use std::borrow::Cow;
use std::fmt::{Display, Formatter, Write};
use std::str::FromStr;

/// ## Xugu Geometric Path type
///
/// Description: Open path or Closed path (similar to polygon)
/// Representation: Open `[(x1,y1),...]`, Closed `((x1,y1),...)`
///
/// Paths are represented by lists of connected points. Paths can be open, where the first and last points in the list are considered not connected, or closed, where the first and last points are considered connected.
/// Values of type path are specified using any of the following syntaxes:
/// ```text
/// [ ( x1 , y1 ) , ... , ( xn , yn ) ]
/// ( ( x1 , y1 ) , ... , ( xn , yn ) )
///   ( x1 , y1 ) , ... , ( xn , yn )
///   ( x1 , y1   , ... ,   xn , yn )
///     x1 , y1   , ... ,   xn , yn
/// ```
/// where the points are the end points of the line segments comprising the path. Square brackets `([])` indicate an open path, while parentheses `(())` indicate a closed path.
/// When the outermost parentheses are omitted, as in the third through fifth syntaxes, a closed path is assumed.
///
#[derive(Debug, Clone, PartialEq)]
pub struct XgPath {
    pub closed: bool,
    pub points: Vec<XgPoint>,
}

impl Type<Xugu> for XgPath {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::PATH)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(ty.r#type, ColumnType::CHAR | ColumnType::PATH)
    }
}

impl<'r> Decode<'r, Xugu> for XgPath {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = value.as_str()?;
        Ok(Self::from_str(s)?)
    }
}

impl Encode<'_, Xugu> for XgPath {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let s = self.to_string();

        args.push(XuguArgumentValue::Str(Cow::Owned(s)));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl Display for XgPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char(if self.closed { '(' } else { '[' })?;
        for (i, p) in self.points.iter().enumerate() {
            if i > 0 {
                f.write_char(',')?;
            }
            Display::fmt(&p, f)?;
        }
        f.write_char(if self.closed { ')' } else { ']' })
    }
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.parse().map_err(|_| Error::Decode(error_msg.into()))
}

impl FromStr for XgPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let closed = !s.contains('[');
        let sanitised = s.replace(['(', ')', '[', ']', ' '], "");
        let parts = sanitised.split(',').collect::<Vec<_>>();

        let mut points = vec![];

        if parts.len() % 2 != 0 {
            return Err(Error::Decode(
                format!("Unmatched pair in PATH: {}", s).into(),
            ));
        }

        for chunk in parts.chunks_exact(2) {
            if let [x_str, y_str] = chunk {
                let x = parse_float_from_str(x_str, "could not get x")?;
                let y = parse_float_from_str(y_str, "could not get y")?;

                let point = XgPoint { x, y };
                points.push(point);
            }
        }

        if !points.is_empty() {
            return Ok(Self { points, closed });
        }

        Err(Error::Decode(
            format!("could not get path from {}", s).into(),
        ))
    }
}

impl XgPath {
    pub fn is_closed(&self) -> bool {
        self.closed
    }
    pub fn is_open(&self) -> bool {
        !self.closed
    }

    pub fn close_path(&mut self) {
        self.closed = true;
    }

    pub fn open_path(&mut self) {
        self.closed = false;
    }
}
