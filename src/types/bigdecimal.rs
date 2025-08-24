use bigdecimal::BigDecimal;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;

use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};

impl Type<Xugu> for BigDecimal {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::NUMERIC)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(ty.r#type, ColumnType::NUMERIC | ColumnType::CHAR)
    }
}

impl<'q> Encode<'q, Xugu> for BigDecimal {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        args.push(XuguArgumentValue::Str(Cow::Owned(self.to_string())));

        Ok(IsNull::No)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl Decode<'_, Xugu> for BigDecimal {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        Ok(value.as_str()?.parse()?)
    }
}
