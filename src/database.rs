use crate::column::XuguColumn;
use crate::connection::XuguConnection;
use crate::type_info::XuguTypeInfo;
use crate::value::{XuguValue, XuguValueRef};
use crate::{XuguArguments, XuguQueryResult, XuguRow, XuguStatement, XuguTransactionManager};
use sqlx_core::database::{Database, HasStatementCache};
use crate::arguments::XuguArgumentValue;

/// Xugu database driver.
#[derive(Debug)]
pub struct Xugu;

impl Database for Xugu {
    type Connection = XuguConnection;

    type TransactionManager = XuguTransactionManager;

    type Row = XuguRow;

    type QueryResult = XuguQueryResult;

    type Column = XuguColumn;

    type TypeInfo = XuguTypeInfo;

    type Value = XuguValue;
    type ValueRef<'r> = XuguValueRef<'r>;

    type Arguments<'q> = XuguArguments<'q>;
    type ArgumentBuffer<'q> = Vec<XuguArgumentValue<'q>>;

    type Statement<'q> = XuguStatement<'q>;

    const NAME: &'static str = "Xugu";

    const URL_SCHEMES: &'static [&'static str] = &["xugu"];
}

impl HasStatementCache for Xugu {}
