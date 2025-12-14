use crate::arguments::XuguArgumentValue;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::{Xugu, XuguTypeInfo, XuguValueRef};

use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use sqlx_core::value::ValueRef;
use std::borrow::Cow;

impl Type<Xugu> for str {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo {
            r#type: ColumnType::CHAR,
            flags: ColumnFlags::empty(),
        }
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::BLOB
                | ColumnType::BLOB_I
                | ColumnType::BLOB_M
                | ColumnType::BLOB_OM
                | ColumnType::BLOB_S
                | ColumnType::CLOB
                | ColumnType::BINARY
                | ColumnType::NUMERIC
                | ColumnType::TINYINT
                | ColumnType::SMALLINT
                | ColumnType::INTEGER
                | ColumnType::BIGINT
                | ColumnType::BOOLEAN
                | ColumnType::CHAR
                | ColumnType::NCHAR
                | ColumnType::GUID
                | ColumnType::ROWID
                | ColumnType::ROWVERSION
                | ColumnType::ARRAY

                // 几何类型 按字符串编解码
                | ColumnType::POINT
                | ColumnType::LSEG
                | ColumnType::PATH
                | ColumnType::BOX
                | ColumnType::POLYGON
                | ColumnType::LINE
                | ColumnType::CIRCLE
                | ColumnType::GEOMETRY
                | ColumnType::GEOGRAPHY
                | ColumnType::BOX2D
                | ColumnType::BOX3D
                | ColumnType::SPHEROID
                | ColumnType::RASTER
        )
    }
}

impl<'q> Encode<'q, Xugu> for &'q str {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(Cow::Borrowed(*self)));

        Ok(IsNull::No)
    }
}

impl<'q> Decode<'q, Xugu> for &'q str {
    fn decode(value: XuguValueRef<'q>) -> Result<Self, BoxDynError> {
        value.as_str()
    }
}

impl Type<Xugu> for Box<str> {
    fn type_info() -> XuguTypeInfo {
        <&str as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <&str as Type<Xugu>>::compatible(ty)
    }
}

impl Encode<'_, Xugu> for Box<str> {
    fn encode(self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(Cow::Owned(self.into_string())));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(Cow::Owned(
            self.clone().into_string(),
        )));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for Box<str> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info().r#type;
        if matches!(
            ty,
            ColumnType::TINYINT | ColumnType::SMALLINT | ColumnType::INTEGER | ColumnType::BIGINT
        ) {
            let num = <i64 as Decode<Xugu>>::decode(value)?;
            return Ok(Box::from(num.to_string()));
        }

        value.as_str().map(Box::from)
    }
}

impl Type<Xugu> for String {
    fn type_info() -> XuguTypeInfo {
        <str as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <str as Type<Xugu>>::compatible(ty)
    }
}

impl Encode<'_, Xugu> for String {
    fn encode(self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(Cow::Owned(self)));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(Cow::Owned(self.clone())));

        Ok(IsNull::No)
    }
}

impl Decode<'_, Xugu> for String {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        let ty = value.type_info().r#type;
        if matches!(
            ty,
            ColumnType::TINYINT | ColumnType::SMALLINT | ColumnType::INTEGER | ColumnType::BIGINT
        ) {
            let num = <i64 as Decode<Xugu>>::decode(value)?;
            return Ok(num.to_string());
        }

        value.as_str().map(ToOwned::to_owned)
    }
}

impl Type<Xugu> for Cow<'_, str> {
    fn type_info() -> XuguTypeInfo {
        <&str as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <&str as Type<Xugu>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Xugu> for Cow<'q, str> {
    fn encode(self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(self));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(self.clone()));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for Cow<'r, str> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info().r#type;
        if matches!(
            ty,
            ColumnType::TINYINT | ColumnType::SMALLINT | ColumnType::INTEGER | ColumnType::BIGINT
        ) {
            let num = <i64 as Decode<Xugu>>::decode(value)?;
            return Ok(Cow::Owned(num.to_string()));
        }

        value.as_str().map(Cow::Borrowed)
    }
}
