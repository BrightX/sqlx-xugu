use crate::arguments::XuguArgumentValue;
use crate::error::BoxDynError;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use byteorder::{BigEndian, ByteOrder};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::types::Type;
use std::borrow::Cow;

fn uint_compatible(ty: &XuguTypeInfo) -> bool {
    matches!(
        ty.r#type,
        ColumnType::TINYINT
            | ColumnType::SMALLINT
            | ColumnType::INTEGER
            | ColumnType::BIGINT
            | ColumnType::BOOLEAN
    ) && !ty.flags.contains(ColumnFlags::IS_LOB)
}

impl Type<Xugu> for u8 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::TINYINT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        uint_compatible(ty)
    }
}

impl Type<Xugu> for u16 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::SMALLINT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        uint_compatible(ty)
    }
}

impl Type<Xugu> for u32 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::INTEGER)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        uint_compatible(ty)
    }
}

impl Type<Xugu> for u64 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::BIGINT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        uint_compatible(ty)
    }
}

impl Encode<'_, Xugu> for u8 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Xugu> for u16 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Xugu> for u32 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Xugu> for u64 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

fn uint_decode(value: XuguValueRef<'_>) -> Result<u64, BoxDynError> {
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
        return <bool as Decode<Xugu>>::decode(value).map(|x| x as u64);
    }

    Ok(BigEndian::read_uint(buf, buf.len()))
}

impl Decode<'_, Xugu> for u8 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        uint_decode(value)?.try_into().map_err(Into::into)
    }
}

impl Decode<'_, Xugu> for u16 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        uint_decode(value)?.try_into().map_err(Into::into)
    }
}

impl Decode<'_, Xugu> for u32 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        uint_decode(value)?.try_into().map_err(Into::into)
    }
}

impl Decode<'_, Xugu> for u64 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        uint_decode(value)
    }
}
