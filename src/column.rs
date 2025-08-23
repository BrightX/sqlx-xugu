use crate::protocol::text::ColumnFlags;
use crate::type_info::XuguTypeInfo;
use crate::Xugu;
pub(crate) use sqlx_core::column::*;
use sqlx_core::ext::ustr::UStr;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct XuguColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: XuguTypeInfo,

    #[cfg_attr(feature = "offline", serde(skip))]
    pub(crate) flags: Option<ColumnFlags>,
}

impl Column for XuguColumn {
    type Database = Xugu;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn type_info(&self) -> &XuguTypeInfo {
        &self.type_info
    }
}
