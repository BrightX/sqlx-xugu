use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::{Text, Type};
use std::fmt::Display;
use std::str::FromStr;
use crate::arguments::XuguArgumentValue;

impl<T> Type<Xugu> for Text<T> {
    fn type_info() -> XuguTypeInfo {
        <String as Type<Xugu>>::type_info()
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        <String as Type<Xugu>>::compatible(ty)
    }
}

impl<'q, T> Encode<'q, Xugu> for Text<T>
where
    T: Display,
{
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'q>>) -> Result<IsNull, BoxDynError> {
        Encode::<Xugu>::encode(self.0.to_string(), args)
    }
}

impl<'r, T> Decode<'r, Xugu> for Text<T>
where
    T: FromStr,
    BoxDynError: From<<T as FromStr>::Err>,
{
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let s: &str = Decode::<Xugu>::decode(value)?;
        Ok(Self(s.parse()?))
    }
}
