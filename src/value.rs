use sqlx_core::bytes::Bytes;
pub(crate) use sqlx_core::value::*;
use std::borrow::Cow;
use std::str::from_utf8;

use crate::error::{BoxDynError, UnexpectedNullError};
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo};

/// Implementation of [`Value`] for Xugu.
#[derive(Clone)]
pub struct XuguValue {
    value: Option<Bytes>,
    type_info: XuguTypeInfo,
}

/// Implementation of [`ValueRef`] for Xugu.
#[derive(Clone)]
pub struct XuguValueRef<'r> {
    pub(crate) value: Option<&'r [u8]>,
    pub(crate) row: Option<&'r Vec<Bytes>>,
    pub(crate) type_info: XuguTypeInfo,
}

impl<'r> XuguValueRef<'r> {
    pub(crate) fn as_bytes(&self) -> Result<&'r [u8], BoxDynError> {
        match &self.value {
            Some(v) => Ok(v),
            None => Err(UnexpectedNullError.into()),
        }
    }

    pub(crate) fn as_str(&self) -> Result<&'r str, BoxDynError> {
        Ok(from_utf8(self.as_bytes()?)?)
    }
}

impl Value for XuguValue {
    type Database = Xugu;

    fn as_ref(&self) -> XuguValueRef<'_> {
        XuguValueRef {
            value: self.value.as_deref(),
            row: None,
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, XuguTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        is_null(self.value.as_deref(), &self.type_info)
    }
}

/// Returns a slice of self that is equivalent to the given `subset`.
///
/// When processing a `Bytes` buffer with other tools, one often gets a
/// `&[u8]` which is in fact a slice of the `Bytes`, i.e. a subset of it.
/// This function turns that `&[u8]` into another `Bytes`, as if one had
/// called `self.slice()` with the offsets that correspond to `subset`.
///
/// This operation is `O(1)`.
///
/// see [`Bytes::slice_ref`]
fn slice_ref(bytes: &Bytes, subset: &[u8]) -> Option<Bytes> {
    // Empty slice and empty Bytes may have their pointers reset
    // so explicitly allow empty slice to be a subslice of any slice.
    if subset.is_empty() {
        return Some(Bytes::new());
    }

    let bytes_p = bytes.as_ptr() as usize;
    let bytes_len = bytes.len();

    let sub_p = subset.as_ptr() as usize;
    let sub_len = subset.len();

    // subset pointer is smaller than self pointer
    if sub_p < bytes_p {
        return None;
    }

    // subset is out of bounds
    if sub_p + sub_len > bytes_p + bytes_len {
        return None;
    }

    let sub_offset = sub_p - bytes_p;

    let sub = bytes.slice(sub_offset..(sub_offset + sub_len));
    Some(sub)
}

fn slice_ref_or_copy(row: &[Bytes], value: &[u8]) -> Bytes {
    for x in row {
        if let Some(v) = slice_ref(x, value) {
            return v;
        }
    }

    Bytes::copy_from_slice(value)
}

impl<'r> ValueRef<'r> for XuguValueRef<'r> {
    type Database = Xugu;

    fn to_owned(&self) -> XuguValue {
        let value = match (self.row, self.value) {
            (Some(row), Some(value)) => Some(slice_ref_or_copy(row, value)),

            (None, Some(value)) => Some(Bytes::copy_from_slice(value)),

            _ => None,
        };

        XuguValue {
            value,
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, XuguTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    #[inline]
    fn is_null(&self) -> bool {
        is_null(self.value, &self.type_info)
    }
}

fn is_null(value: Option<&[u8]>, ty: &XuguTypeInfo) -> bool {
    if let Some(value) = value {
        if value.is_empty() {
            return true;
        }
        // zero dates and date times should be treated the same as NULL
        if matches!(
            ty.r#type,
            ColumnType::DATE | ColumnType::TIME | ColumnType::DATETIME
        ) && value.starts_with(b"\0")
        {
            return true;
        }
    }

    value.is_none()
}
