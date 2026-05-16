use crate::protocol::text::ColumnType;
use crate::{
    Xugu, XuguArguments, XuguColumn, XuguConnectOptions, XuguConnection, XuguQueryResult, XuguRow,
    XuguTransactionManager, XuguTypeInfo,
};
use futures_core::future::BoxFuture;
use futures_core::stream::BoxStream;
use futures_util::{stream, StreamExt, TryFutureExt, TryStreamExt};
use sqlx_core::any::{
    Any, AnyArguments, AnyColumn, AnyConnectOptions, AnyConnectionBackend, AnyQueryResult, AnyRow,
    AnyStatement, AnyTypeInfo, AnyTypeInfoKind, AnyValueKind,
};
use sqlx_core::arguments::Arguments;
use sqlx_core::connection::Connection;
use sqlx_core::database::Database;
use sqlx_core::describe::Describe;
use sqlx_core::executor::Executor;
use sqlx_core::transaction::TransactionManager;
use sqlx_core::Either;
use std::borrow::Cow;
use std::future;
use std::pin::pin;

sqlx_core::declare_driver_with_optional_migrate!(DRIVER = Xugu);

impl AnyConnectionBackend for XuguConnection {
    fn name(&self) -> &str {
        <Xugu as Database>::NAME
    }

    fn close(self: Box<Self>) -> BoxFuture<'static, sqlx_core::Result<()>> {
        Connection::close(*self)
    }

    fn close_hard(self: Box<Self>) -> BoxFuture<'static, sqlx_core::Result<()>> {
        Connection::close_hard(*self)
    }

    fn ping(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        Connection::ping(self)
    }

    fn begin(
        &mut self,
        statement: Option<Cow<'static, str>>,
    ) -> BoxFuture<'_, sqlx_core::Result<()>> {
        XuguTransactionManager::begin(self, statement)
    }

    fn commit(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        XuguTransactionManager::commit(self)
    }

    fn rollback(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        XuguTransactionManager::rollback(self)
    }

    fn start_rollback(&mut self) {
        XuguTransactionManager::start_rollback(self)
    }

    fn get_transaction_depth(&self) -> usize {
        XuguTransactionManager::get_transaction_depth(self)
    }

    fn shrink_buffers(&mut self) {
        Connection::shrink_buffers(self)
    }

    fn flush(&mut self) -> BoxFuture<'_, sqlx_core::Result<()>> {
        Connection::flush(self)
    }

    fn should_flush(&self) -> bool {
        Connection::should_flush(self)
    }

    #[cfg(feature = "migrate")]
    fn as_migrate(
        &mut self,
    ) -> sqlx_core::Result<&mut (dyn sqlx_core::migrate::Migrate + Send + 'static)> {
        unimplemented!("as_migrate() is not implemented for sqlx-xugu.")
        // Ok(self)
    }

    fn fetch_many<'q>(
        &'q mut self,
        query: &'q str,
        persistent: bool,
        arguments: Option<AnyArguments<'q>>,
    ) -> BoxStream<'q, sqlx_core::Result<Either<AnyQueryResult, AnyRow>>> {
        let persistent = persistent && arguments.is_some();
        let args = match arguments.map(map_arguments).transpose() {
            Ok(arguments) => arguments,
            Err(error) => return stream::once(future::ready(Err(error))).boxed(),
        };

        Box::pin(self.run(query, args, persistent).try_flatten_stream().map(
            move |res: sqlx_core::Result<Either<XuguQueryResult, XuguRow>>| match res? {
                Either::Left(result) => Ok(Either::Left(map_result(result))),
                Either::Right(row) => Ok(Either::Right(AnyRow::try_from(&row)?)),
            },
        ))
    }

    fn fetch_optional<'q>(
        &'q mut self,
        query: &'q str,
        persistent: bool,
        arguments: Option<AnyArguments<'q>>,
    ) -> BoxFuture<'q, sqlx_core::Result<Option<AnyRow>>> {
        let persistent = persistent && arguments.is_some();
        let args = arguments.map(map_arguments).transpose();

        Box::pin(async move {
            let args = args?;
            let mut stream = pin!(self.run(query, args, persistent).await?);

            if let Some(Either::Right(row)) = stream.try_next().await? {
                return Ok(Some(AnyRow::try_from(&row)?));
            }

            Ok(None)
        })
    }

    fn prepare_with<'c, 'q: 'c>(
        &'c mut self,
        sql: &'q str,
        _parameters: &[AnyTypeInfo],
    ) -> BoxFuture<'c, sqlx_core::Result<AnyStatement<'q>>> {
        Box::pin(async move {
            let statement = Executor::prepare_with(self, sql, &[]).await?;

            AnyStatement::try_from_statement(
                sql,
                &statement,
                statement.metadata.column_names.clone(),
            )
        })
    }

    fn describe<'q>(&'q mut self, sql: &'q str) -> BoxFuture<'q, sqlx_core::Result<Describe<Any>>> {
        Box::pin(async move {
            let describe = Executor::describe(self, sql).await?;

            let columns = describe
                .columns
                .iter()
                .map(AnyColumn::try_from)
                .collect::<Result<Vec<_>, _>>()?;

            let parameters = match describe.parameters {
                Some(Either::Left(parameters)) => Some(Either::Left(
                    parameters
                        .iter()
                        .enumerate()
                        .map(|(i, type_info)| {
                            AnyTypeInfo::try_from(type_info).map_err(|_| {
                                sqlx_core::Error::AnyDriverError(
                                    format!(
                                        "Any driver does not support type {type_info} of parameter {i}"
                                    )
                                    .into(),
                                )
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )),
                Some(Either::Right(count)) => Some(Either::Right(count)),
                None => None,
            };

            Ok(Describe {
                columns,
                parameters,
                nullable: describe.nullable,
            })
        })
    }
}

impl<'a> TryFrom<&'a XuguTypeInfo> for AnyTypeInfo {
    type Error = sqlx_core::Error;

    fn try_from(xg_type: &'a XuguTypeInfo) -> Result<Self, Self::Error> {
        Ok(AnyTypeInfo {
            kind: match &xg_type.r#type {
                ColumnType::NONE | ColumnType::NULL => AnyTypeInfoKind::Null,
                ColumnType::BOOLEAN => AnyTypeInfoKind::Bool,
                ColumnType::TINYINT => AnyTypeInfoKind::SmallInt,
                ColumnType::SMALLINT => AnyTypeInfoKind::SmallInt,
                ColumnType::INTEGER => AnyTypeInfoKind::Integer,
                ColumnType::BIGINT => AnyTypeInfoKind::BigInt,
                ColumnType::NUMERIC => AnyTypeInfoKind::Text,
                ColumnType::FLOAT => AnyTypeInfoKind::Real,
                ColumnType::DOUBLE => AnyTypeInfoKind::Double,
                ColumnType::ROWVERSION => AnyTypeInfoKind::Text,
                ColumnType::GUID => AnyTypeInfoKind::Text,
                ColumnType::CHAR => AnyTypeInfoKind::Text,
                ColumnType::NCHAR => AnyTypeInfoKind::Text,
                ColumnType::CLOB => AnyTypeInfoKind::Text,
                ColumnType::BLOB => AnyTypeInfoKind::Blob,
                ColumnType::BLOB_I => AnyTypeInfoKind::Blob,
                ColumnType::BLOB_S => AnyTypeInfoKind::Blob,
                ColumnType::BLOB_M => AnyTypeInfoKind::Blob,
                ColumnType::BLOB_OM => AnyTypeInfoKind::Blob,
                ColumnType::ROWID => AnyTypeInfoKind::Text,
                _ => {
                    return Err(sqlx_core::Error::AnyDriverError(
                        format!("Any driver does not support the Xugu type {xg_type:?}").into(),
                    ))
                }
            },
        })
    }
}

impl<'a> TryFrom<&'a XuguColumn> for AnyColumn {
    type Error = sqlx_core::Error;

    fn try_from(col: &'a XuguColumn) -> Result<Self, Self::Error> {
        let type_info =
            AnyTypeInfo::try_from(&col.type_info).map_err(|e| sqlx_core::Error::ColumnDecode {
                index: col.name.to_string(),
                source: e.into(),
            })?;

        Ok(AnyColumn {
            ordinal: col.ordinal,
            name: col.name.clone(),
            type_info,
        })
    }
}

impl<'a> TryFrom<&'a XuguRow> for AnyRow {
    type Error = sqlx_core::Error;

    fn try_from(row: &'a XuguRow) -> Result<Self, Self::Error> {
        AnyRow::map_from(row, row.column_names.clone())
    }
}

impl<'a> TryFrom<&'a AnyConnectOptions> for XuguConnectOptions {
    type Error = sqlx_core::Error;

    fn try_from(value: &'a AnyConnectOptions) -> Result<Self, Self::Error> {
        let mut opts = XuguConnectOptions::parse_from_url(&value.database_url)?;
        opts.log_settings = value.log_settings.clone();
        Ok(opts)
    }
}

/// Instead of `AnyArguments::convert_into()`, we can do a direct mapping and preserve the lifetime.
fn map_arguments(args: AnyArguments<'_>) -> sqlx_core::Result<XuguArguments<'_>> {
    let mut out = XuguArguments::default();
    out.reserve(args.len(), args.len());

    for arg in args.values.0 {
        match arg {
            AnyValueKind::Null(AnyTypeInfoKind::Null) => out.add(Option::<i32>::None),
            AnyValueKind::Null(AnyTypeInfoKind::Bool) => out.add(Option::<bool>::None),
            AnyValueKind::Null(AnyTypeInfoKind::SmallInt) => out.add(Option::<i16>::None),
            AnyValueKind::Null(AnyTypeInfoKind::Integer) => out.add(Option::<i32>::None),
            AnyValueKind::Null(AnyTypeInfoKind::BigInt) => out.add(Option::<i64>::None),
            AnyValueKind::Null(AnyTypeInfoKind::Real) => out.add(Option::<f64>::None),
            AnyValueKind::Null(AnyTypeInfoKind::Double) => out.add(Option::<f32>::None),
            AnyValueKind::Null(AnyTypeInfoKind::Text) => out.add(Option::<String>::None),
            AnyValueKind::Null(AnyTypeInfoKind::Blob) => out.add(Option::<Vec<u8>>::None),
            AnyValueKind::Bool(b) => out.add(b),
            AnyValueKind::SmallInt(i) => out.add(i),
            AnyValueKind::Integer(i) => out.add(i),
            AnyValueKind::BigInt(i) => out.add(i),
            AnyValueKind::Real(r) => out.add(r),
            AnyValueKind::Double(d) => out.add(d),
            AnyValueKind::Text(t) => out.add(t),
            AnyValueKind::Blob(b) => out.add(b),
            // AnyValueKind is `#[non_exhaustive]` but we should have covered everything
            _ => unreachable!("BUG: missing mapping for {arg:?}"),
        }
        .map_err(sqlx_core::Error::Encode)?;
    }
    Ok(out)
}

fn map_result(res: XuguQueryResult) -> AnyQueryResult {
    AnyQueryResult {
        rows_affected: res.rows_affected(),
        last_insert_id: None,
    }
}
