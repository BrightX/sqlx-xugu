use sqlx_core::bytes::Bytes;
use sqlx_core::ext::ustr::UStr;
pub(crate) use sqlx_core::row::*;
use sqlx_core::HashMap;
use std::sync::Arc;

use crate::column::{ColumnIndex, XuguColumn};
use crate::error::Error;
use crate::{Xugu, XuguValueRef};

/// Implementation of [`Row`] for Xugu.
#[derive(Debug)]
pub struct XuguRow {
    pub(crate) row: Arc<Vec<Bytes>>,
    pub(crate) columns: Arc<Vec<XuguColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
}

impl Row for XuguRow {
    type Database = Xugu;

    fn columns(&self) -> &[XuguColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<XuguValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let column = &self.columns[index];
        let value = self.row.get(index).map(|x| x.as_ref());

        Ok(XuguValueRef {
            row: Some(&self.row),
            type_info: column.type_info.clone(),
            value,
        })
    }
}

impl ColumnIndex<XuguRow> for &'_ str {
    fn index(&self, row: &XuguRow) -> Result<usize, Error> {
        row.column_names
            .get(*self)
            .or_else(|| {
                // 列名忽略大小写时，列名大写检索
                row.column_names.get(&*self.to_uppercase())
            })
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .copied()
    }
}
