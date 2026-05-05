use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use bytes::Bytes;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::{BoxDynError, UnexpectedNullError};
use sqlx_core::types::Type;
use sqlx_core::value::ValueRef;
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
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(self)));
        }

        Ok(IsNull::No)
    }
}

impl<'q> Decode<'q, Xugu> for &'q [u8] {
    fn decode(value: XuguValueRef<'q>) -> Result<Self, BoxDynError> {
        value.as_bytes().map(map_empty)
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
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bin(Cow::Owned(self.into_vec())));
        }

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bin(Cow::Owned(self.clone().into_vec())));
        }

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for Box<[u8]> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        value.as_bytes().map(map_empty).map(Box::from)
    }
}

impl Type<Xugu> for Cow<'_, [u8]> {
    fn type_info() -> XuguTypeInfo {
        <&[u8] as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <&[u8] as Type<Xugu>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Xugu> for Cow<'q, [u8]> {
    fn encode(self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bin(self));
        }

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bin(self.clone()));
        }

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for Cow<'r, [u8]> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        value.as_bytes().map(map_empty).map(Cow::Borrowed)
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
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bin(Cow::Owned(self)));
        }

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bin(Cow::Owned(self.clone())));
        }

        Ok(IsNull::No)
    }
}

impl Decode<'_, Xugu> for Vec<u8> {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        value.as_bytes().map(map_empty).map(ToOwned::to_owned)
    }
}

impl Type<Xugu> for Bytes {
    fn type_info() -> XuguTypeInfo {
        <&[u8] as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <&[u8] as Type<Xugu>>::compatible(ty)
    }
}

impl Encode<'_, Xugu> for Bytes {
    fn encode(self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bytes(self));
        }

        Ok(IsNull::No)
    }

    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        if self.is_empty() {
            args.push(XuguArgumentValue::Bin(Cow::Borrowed(b"\0")));
        } else {
            args.push(XuguArgumentValue::Bytes(self.clone()));
        }

        Ok(IsNull::No)
    }
}

impl Decode<'_, Xugu> for Bytes {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        let v = ValueRef::to_owned(&value);
        match v.value {
            Some(v) => {
                if b"\0".as_slice() == v {
                    return Ok(Bytes::new());
                }
                Ok(v)
            }
            None => Err(UnexpectedNullError.into()),
        }
    }
}

// 处理空字节
fn map_empty(s: &[u8]) -> &[u8] {
    if s == b"\0" {
        return b"";
    }
    s
}
