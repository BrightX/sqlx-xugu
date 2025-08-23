use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};

use serde::{Deserialize, Serialize};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::{Json, Type};

impl<T> Type<Xugu> for Json<T> {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::JSON)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(ty.r#type, ColumnType::CHAR | ColumnType::JSON)
    }
}

impl<T> Encode<'_, Xugu> for Json<T>
where
    T: Serialize,
{
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        Encode::<Xugu>::encode(self.encode_to_string()?, args)
    }

    fn produces(&self) -> Option<XuguTypeInfo> {
        Some(XuguTypeInfo::binary(ColumnType::CHAR))
    }
}

impl<'r, T> Decode<'r, Xugu> for Json<T>
where
    T: 'r + Deserialize<'r>,
{
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        Self::decode_from_string(Decode::<Xugu>::decode(value)?)
    }
}
