use crate::arguments::XuguArgumentValue;
use crate::error::BoxDynError;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use byteorder::{BigEndian, ByteOrder};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::types::Type;
use std::borrow::Cow;

fn int_compatible(ty: &XuguTypeInfo) -> bool {
    matches!(
        ty.r#type,
        ColumnType::TINYINT
            | ColumnType::SMALLINT
            | ColumnType::INTEGER
            | ColumnType::BIGINT
            | ColumnType::BOOLEAN
    ) && !ty.flags.contains(ColumnFlags::IS_LOB)
}

impl Type<Xugu> for i8 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::TINYINT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Type<Xugu> for i16 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::SMALLINT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Type<Xugu> for i32 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::INTEGER)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Type<Xugu> for i64 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::BIGINT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        int_compatible(ty)
    }
}

impl Encode<'_, Xugu> for i8 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Xugu> for i16 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Xugu> for i32 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Xugu> for i64 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

fn int_decode(value: XuguValueRef<'_>) -> Result<i64, BoxDynError> {
    let buf = value.as_bytes()?;

    // Check conditions that could cause `read_int()` to panic.
    if buf.is_empty() {
        return Err("empty buffer".into());
    }

    if buf.len() > 8 {
        return Err(format!(
            "expected no more than 8 bytes for integer value, got {}",
            buf.len()
        )
        .into());
    }

    if value.type_info.r#type == ColumnType::BOOLEAN {
        return <bool as Decode<Xugu>>::decode(value).map(|x| x as i64);
    }

    Ok(BigEndian::read_int(buf, buf.len()))
}

impl Decode<'_, Xugu> for i8 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        int_decode(value)?.try_into().map_err(Into::into)
    }
}

impl Decode<'_, Xugu> for i16 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        int_decode(value)?.try_into().map_err(Into::into)
    }
}

impl Decode<'_, Xugu> for i32 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        int_decode(value)?.try_into().map_err(Into::into)
    }
}

impl Decode<'_, Xugu> for i64 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        int_decode(value)
    }
}
