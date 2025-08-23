use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;

impl Type<Xugu> for [u8] {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::BLOB)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::BLOB
                | ColumnType::BLOB_I
                | ColumnType::BLOB_M
                | ColumnType::BLOB_OM
                | ColumnType::BLOB_S
                | ColumnType::CHAR
                | ColumnType::NCHAR
                | ColumnType::CLOB
                | ColumnType::BINARY
                | ColumnType::GUID
        )
    }
}

impl<'q> Encode<'q, Xugu> for &'q [u8] {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Bin(Cow::Borrowed(self)));

        Ok(IsNull::No)
    }
}

impl<'q> Decode<'q, Xugu> for &'q [u8] {
    fn decode(value: XuguValueRef<'q>) -> Result<Self, BoxDynError> {
        value.as_bytes()
    }
}

impl Type<Xugu> for Box<[u8]> {
    fn type_info() -> XuguTypeInfo {
        <&[u8] as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <&[u8] as Type<Xugu>>::compatible(ty)
    }
}

impl Encode<'_, Xugu> for Box<[u8]> {
    fn encode(self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Bin(Cow::Owned(self.into_vec())));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Bin(Cow::Owned(self.clone().into_vec())));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for Box<[u8]> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        value.as_bytes().map(Box::from)
    }
}

impl Type<Xugu> for Vec<u8> {
    fn type_info() -> XuguTypeInfo {
        <[u8] as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <&[u8] as Type<Xugu>>::compatible(ty)
    }
}

impl Encode<'_, Xugu> for Vec<u8> {
    fn encode(self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Bin(Cow::Owned(self)));

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Bin(Cow::Owned(self.clone())));

        Ok(IsNull::No)
    }
}

impl Decode<'_, Xugu> for Vec<u8> {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        value.as_bytes().map(ToOwned::to_owned)
    }
}
