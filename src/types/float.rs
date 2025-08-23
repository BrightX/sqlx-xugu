use crate::arguments::XuguArgumentValue;
use crate::error::BoxDynError;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use byteorder::{BigEndian, ByteOrder};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::types::Type;
use std::borrow::Cow;

fn real_compatible(ty: &XuguTypeInfo) -> bool {
    // NOTE: `DECIMAL` is explicitly excluded because floating-point numbers have different semantics.
    matches!(ty.r#type, ColumnType::FLOAT | ColumnType::DOUBLE)
}

impl Type<Xugu> for f32 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::FLOAT)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        real_compatible(ty)
    }
}

impl Type<Xugu> for f64 {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::DOUBLE)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        real_compatible(ty)
    }
}

impl Encode<'_, Xugu> for f32 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Encode<'_, Xugu> for f64 {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue<'_>>) -> Result<IsNull, BoxDynError> {
        let buf = self.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl Decode<'_, Xugu> for f32 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        let buf = value.as_bytes()?;

        Ok(match buf.len() {
            // These functions panic if `buf` is not exactly the right size.
            4 => BigEndian::read_f32(buf),
            // Xugu can return 8-byte DOUBLE values for a FLOAT
            // We take and truncate to f32 as that's the same behavior as *in* Xugu,
            #[allow(clippy::cast_possible_truncation)]
            8 => BigEndian::read_f64(buf) as f32,
            other => {
                // Users may try to decode a DECIMAL as floating point;
                // inform them why that's a bad idea.
                return Err(format!(
                    "expected a FLOAT as 4 or 8 bytes, got {other} bytes; \
                             note that decoding DECIMAL as `f32` is not supported \
                             due to differing semantics"
                )
                .into());
            }
        })
    }
}

impl Decode<'_, Xugu> for f64 {
    fn decode(value: XuguValueRef<'_>) -> Result<Self, BoxDynError> {
        let buf = value.as_bytes()?;

        // The `read_*` functions panic if `buf` is not exactly the right size.
        Ok(match buf.len() {
            // Allow implicit widening here
            4 => BigEndian::read_f32(buf) as f64,
            8 => BigEndian::read_f64(buf),
            other => {
                // Users may try to decode a DECIMAL as floating point;
                // inform them why that's a bad idea.
                return Err(format!(
                    "expected a DOUBLE as 4 or 8 bytes, got {other} bytes; \
                             note that decoding DECIMAL as `f64` is not supported \
                             due to differing semantics"
                )
                .into());
            }
        })
    }
}
