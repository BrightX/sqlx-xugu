mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod io;
mod options;
mod protocol;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

pub use arguments::XuguArguments;
pub use column::XuguColumn;
pub use connection::XuguConnection;
pub use database::Xugu;
pub use error::XuguDatabaseError;
pub use options::XuguConnectOptions;
pub use query_result::XuguQueryResult;
pub use row::XuguRow;
use sqlx_core::executor::Executor;
use sqlx_core::transaction::Transaction;
use sqlx_core::{
    impl_acquire, impl_column_index_for_row, impl_column_index_for_statement,
    impl_encode_for_option, impl_into_arguments_for_arguments, pool,
};
pub use statement::XuguStatement;
pub use transaction::XuguTransactionManager;
pub use type_info::XuguTypeInfo;
pub use value::{XuguValue, XuguValueRef};

/// An alias for [`Pool`][sqlx_core::pool::Pool], specialized for Xugu.
pub type XuguPool = pool::Pool<Xugu>;

/// An alias for [`PoolOptions`][sqlx_core::pool::PoolOptions], specialized for Xugu.
pub type XuguPoolOptions = pool::PoolOptions<Xugu>;

/// An alias for [`Executor<'_, Database = Xugu>`][Executor].
pub trait XuguExecutor<'c>: Executor<'c, Database = Xugu> {}
impl<'c, T: Executor<'c, Database = Xugu>> XuguExecutor<'c> for T {}

/// An alias for [`Transaction`][sqlx_core::transaction::Transaction], specialized for Xugu.
pub type XuguTransaction<'c> = Transaction<'c, Xugu>;

// NOTE: required due to the lack of lazy normalization
impl_into_arguments_for_arguments!(XuguArguments<'q>);
impl_acquire!(Xugu, XuguConnection);
impl_column_index_for_row!(XuguRow);
impl_column_index_for_statement!(XuguStatement);

// required because some databases have a different handling of NULL
impl_encode_for_option!(Xugu);

/// Convenience re-export of common traits.
pub mod prelude {
    pub use futures_core::future::BoxFuture;
    pub use futures_core::stream::BoxStream;
    pub use futures_util::TryStreamExt;

    pub use sqlx_core::acquire::Acquire;
    pub use sqlx_core::arguments::Arguments;
    pub use sqlx_core::arguments::IntoArguments;
    pub use sqlx_core::column::Column;
    pub use sqlx_core::column::ColumnIndex;
    pub use sqlx_core::connection::ConnectOptions;
    pub use sqlx_core::connection::Connection;
    pub use sqlx_core::decode::Decode;
    pub use sqlx_core::encode::Encode;
    pub use sqlx_core::error::DatabaseError;
    pub use sqlx_core::error::Error;
    pub use sqlx_core::error::Result;
    pub use sqlx_core::executor::Executor;
    pub use sqlx_core::from_row::FromRow;

    pub use sqlx_core::query::query_with_result as __query_with_result;
    pub use sqlx_core::query::{query, query_with};
    pub use sqlx_core::query_as::{query_as, query_as_with};
    pub use sqlx_core::query_builder::{self, QueryBuilder};
    #[doc(hidden)]
    pub use sqlx_core::query_scalar::query_scalar_with_result as __query_scalar_with_result;
    pub use sqlx_core::query_scalar::{query_scalar, query_scalar_with};
    pub use sqlx_core::raw_sql::{raw_sql, RawSql};

    pub use sqlx_core::row::Row;
    pub use sqlx_core::statement::Statement;
    pub use sqlx_core::type_info::TypeInfo;
    pub use sqlx_core::types::Type;
    pub use sqlx_core::value::Value;
    pub use sqlx_core::value::ValueRef;
}
