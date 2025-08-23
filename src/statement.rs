use super::XuguColumn;
use crate::column::ColumnIndex;
use crate::error::Error;
use crate::{Xugu, XuguArguments, XuguTypeInfo};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::{impl_statement_query, Either, HashMap};
use std::borrow::Cow;
use std::sync::Arc;

use crate::protocol::statement::ParameterDef;
pub(crate) use sqlx_core::statement::*;

#[derive(Debug, Clone)]
pub struct XuguStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) metadata: XuguStatementMetadata,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct XuguStatementMetadata {
    pub(crate) columns: Arc<Vec<XuguColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
    pub(crate) parameters: Arc<Vec<ParameterDef>>,
}

impl<'q> Statement<'q> for XuguStatement<'q> {
    type Database = Xugu;

    fn to_owned(&self) -> XuguStatement<'static> {
        XuguStatement::<'static> {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    /// 获取此语句的预期参数。
    ///
    /// 返回的信息取决于驱动程序提供的信息。SQLite 只能告诉我们参数的数量。PostgreSQL 可以为我们提供完整的类型信息。
    fn parameters(&self) -> Option<Either<&[XuguTypeInfo], usize>> {
        Some(Either::Right(self.metadata.parameters.len()))
    }

    fn columns(&self) -> &[XuguColumn] {
        &self.metadata.columns
    }

    impl_statement_query!(XuguArguments);
}

impl ColumnIndex<XuguStatement<'_>> for &'_ str {
    fn index(&self, statement: &XuguStatement<'_>) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
            .copied()
    }
}
