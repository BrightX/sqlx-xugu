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

/// ## Xugu Geometric Polygon type
///
/// Description: Polygon (similar to closed polygon)
/// Representation: `((x1,y1),...)`
///
/// Polygons are represented by lists of points (the vertexes of the polygon). Polygons are very similar to closed paths; the essential semantic difference is that a polygon is considered to include the area within it, while a path is not.
/// An important implementation difference between polygons and paths is that the stored representation of a polygon includes its smallest bounding box. This speeds up certain search operations, although computing the bounding box adds overhead while constructing new polygons.
/// Values of type polygon are specified using any of the following syntaxes:
///
/// ```text
/// ( ( x1 , y1 ) , ... , ( xn , yn ) )
///   ( x1 , y1 ) , ... , ( xn , yn )
///   ( x1 , y1   , ... ,   xn , yn )
///     x1 , y1   , ... ,   xn , yn
/// ```
///
/// where the points are the end points of the line segments comprising the boundary of the polygon.
///
#[derive(Debug, Clone, PartialEq)]
pub struct XgPolygon {
    pub points: Vec<XgPoint>,
}

impl Type<Xugu> for XgPolygon {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::POLYGON)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::CHAR | ColumnType::POLYGON | ColumnType::POLYGON_OLD
        )
    }
}

impl<'r> Decode<'r, Xugu> for XgPolygon {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = value.as_str()?;
        Ok(Self::from_str(s)?)
    }
}

impl Encode<'_, Xugu> for XgPolygon {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let s = self.to_string();

        args.push(XuguArgumentValue::Str(Cow::Owned(s)));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl Display for XgPolygon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_char('(')?;
        for (i, p) in self.points.iter().enumerate() {
            if i > 0 {
                f.write_char(',')?;
            }
            Display::fmt(&p, f)?;
        }
        f.write_char(')')
    }
}

fn parse_float_from_str(s: &str, error_msg: &str) -> Result<f64, Error> {
    s.parse().map_err(|_| Error::Decode(error_msg.into()))
}

impl FromStr for XgPolygon {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sanitised = s.replace(['(', ')', '[', ']', ' '], "");
        let parts = sanitised.split(',').collect::<Vec<_>>();

        let mut points = vec![];

        if parts.len() % 2 != 0 {
            return Err(Error::Decode(
                format!("Unmatched pair in POLYGON: {}", s).into(),
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
            return Ok(Self { points });
        }

        Err(Error::Decode(
            format!("could not get polygon from {}", s).into(),
        ))
    }
}
