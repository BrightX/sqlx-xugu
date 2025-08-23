use crate::arguments::XuguArgumentValue;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;

impl Type<Xugu> for bool {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo {
            r#type: ColumnType::BOOLEAN,
            flags: ColumnFlags::empty(),
        }
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::BOOLEAN
                | ColumnType::TINYINT
                | ColumnType::SMALLINT
                | ColumnType::INTEGER
                | ColumnType::BIGINT
                | ColumnType::BIT
                | ColumnType::CHAR
                | ColumnType::BINARY
        ) && !ty.flags.contains(ColumnFlags::IS_LOB)
    }
}

impl Encode<'_, Xugu> for bool {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = if *self { b"\x01" } else { b"\x00" };
        args.push(XuguArgumentValue::Bin(Cow::Borrowed(buf)));

        Ok(IsNull::No)
    }
}

impl Decode<'_, Xugu> for bool {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        // 整数类型 转 bool
        match value.type_info.r#type {
            ColumnType::TINYINT
            | ColumnType::SMALLINT
            | ColumnType::INTEGER
            | ColumnType::BIGINT => return <i64 as Decode<Xugu>>::decode(value).map(|x| x != 0),
            _ => {}
        }

        match value.as_bytes()?[0] {
            b'T' | b't' => Ok(true),
            b'1' | 0x01 => Ok(true),
            b'U' | b'u' => Ok(true),

            b'F' | b'f' => Ok(false),
            b'0' | 0x00 => Ok(false),
            s => Err(format!("unexpected value '{}' for boolean", s as char).into()),
        }
    }
}
