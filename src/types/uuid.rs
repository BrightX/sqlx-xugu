use crate::arguments::XuguArgumentValue;
use crate::protocol::text::{ColumnFlags, ColumnType};
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;
use uuid::fmt::{Hyphenated, Simple};
use uuid::Uuid;

impl Type<Xugu> for Uuid {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::GUID)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::GUID | ColumnType::BINARY | ColumnType::CHAR
        ) && !ty.flags.contains(ColumnFlags::IS_LOB)
    }
}

impl<'q> Encode<'q, Xugu> for Uuid {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(Cow::Owned(
            self.as_hyphenated().to_string(),
        )));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl<'r> Decode<'r, Xugu> for Uuid {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        if ty == ColumnType::GUID || ty == ColumnType::CHAR {
            let text = value.as_str()?;
            // parse a UUID from the text
            return Uuid::parse_str(text).map_err(Into::into);
        }

        let bytes = value.as_bytes()?;

        if bytes.len() != 16 {
            return Err(format!(
                "Expected 16 bytes, got {}; `Uuid` uses binary format for Xugu. \
                 For text-formatted UUIDs, use `uuid::fmt::Hyphenated` instead of `Uuid`.",
                bytes.len(),
            )
            .into());
        }

        // construct an Uuid from the returned bytes
        Uuid::from_slice(bytes).map_err(Into::into)
    }
}

impl Type<Xugu> for Hyphenated {
    fn type_info() -> XuguTypeInfo {
        <Uuid as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <Uuid as Type<Xugu>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Xugu> for Hyphenated {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Xugu>::encode_by_ref(self.as_uuid(), args)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        <Uuid as Encode<Xugu>>::produces(self.as_uuid())
    }
}

impl<'r> Decode<'r, Xugu> for Hyphenated {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        <Uuid as Decode<Xugu>>::decode(value).map(|u| u.into())
    }
}

impl Type<Xugu> for Simple {
    fn type_info() -> XuguTypeInfo {
        <Uuid as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <Uuid as Type<Xugu>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Xugu> for Simple {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Xugu>::encode_by_ref(self.as_uuid(), args)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        <Uuid as Encode<Xugu>>::produces(self.as_uuid())
    }
}

impl<'r> Decode<'r, Xugu> for Simple {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        <Uuid as Decode<Xugu>>::decode(value).map(|u| u.into())
    }
}
