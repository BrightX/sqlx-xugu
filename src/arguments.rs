use crate::{Xugu, XuguTypeInfo};
pub(crate) use sqlx_core::arguments::*;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub enum XuguArgumentValue<'q> {
    Null,
    Str(Cow<'q, str>),
    Bin(Cow<'q, [u8]>),
}

/// Implementation of [`Arguments`] for Xugu.
#[derive(Debug, Default, Clone)]
pub struct XuguArguments<'q> {
    pub(crate) values: Vec<XuguArgumentValue<'q>>,
    pub(crate) types: Vec<XuguTypeInfo>,
}

impl<'q> XuguArguments<'q> {
    pub(crate) fn add<T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: Encode<'q, Xugu> + Type<Xugu>,
    {
        let ty = value.produces().unwrap_or_else(T::type_info);

        let value_length_before_encoding = self.values.len();
        match value.encode(&mut self.values) {
            Ok(IsNull::Yes) => self.values.push(XuguArgumentValue::Null),
            Ok(IsNull::No) => {}
            Err(error) => {
                // reset the value buffer to its previous value if encoding failed so we don't leave a half-encoded value behind
                self.values.truncate(value_length_before_encoding);
                return Err(error);
            }
        };

        self.types.push(ty);

        Ok(())
    }
}

impl<'q> Arguments<'q> for XuguArguments<'q> {
    type Database = Xugu;

    fn reserve(&mut self, len: usize, size: usize) {
        self.types.reserve(len);
        self.values.reserve(size);
    }

    fn add<T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: Encode<'q, Self::Database> + Type<Self::Database>,
    {
        self.add(value)
    }

    fn len(&self) -> usize {
        self.types.len()
    }
}
