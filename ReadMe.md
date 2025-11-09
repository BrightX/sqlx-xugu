# sqlx-xugu

> 基于 `sqlx` 的 `rust` 虚谷数据库驱动。
> 
> 支持虚谷数据库 `v11`/`v12`。

## Install

添加依赖，注意版本要跟 `sqlx` 保持一致

```toml
# Cargo.toml
sqlx = { version = "=0.8.6", features = ["runtime-tokio"] }
sqlx-xugu = { version = "=0.8.6" }

```

#### Cargo Feature Flags

添加 feature, 注意要跟 `sqlx` 保持一致

```toml
# Cargo.toml
sqlx = { version = "=0.8.6", features = ["runtime-tokio", "chrono", "uuid", "rust_decimal", "bigdecimal", "time"] }
sqlx-xugu = { version = "=0.8.6", features = ["chrono", "uuid", "rust_decimal", "bigdecimal", "time"] }
```

-   `uuid`: 添加对 UUID 的支持。

-   `chrono`: 添加对 `chrono` 中的日期和时间类型的支持。

-   `time`: 添加对 `time` crate 中的日期和时间类型的支持 (alternative to `chrono`, which is preferred by `query!` macro, if both enabled)

-   `bigdecimal`: 使用 `bigdecimal` crate 添加对 `NUMERIC` 的支持。

-   `rust_decimal`: 使用 `rust_decimal` crate 添加对 `NUMERIC` 的支持。

-   `json`: 使用 `serde_json` crate 添加对 `JSON` 的支持。

## Usage

### 快速入门

```rust
use sqlx_xugu::XuguPoolOptions;

#[async_std::main] // Requires the `attributes` feature of `async-std`
// or #[tokio::main]
// or #[actix_web::main]
async fn main() -> Result<(), sqlx::Error> {
    // Create a connection pool
    //  for Xugu, use XuguPoolOptions::new()
    let pool = XuguPoolOptions::new()
        .max_connections(5)
        .connect("xugu://user:password@127.0.0.1:5138/SYSTEM").await?;

    // 进行简单的查询以返回给定的参数 (使用问号 `?` 占位符)
    let row: (i64,) = sqlx::query_as("SELECT ?")
        .bind(150_i64)
        .fetch_one(&pool).await?;

    assert_eq!(row.0, 150);

    Ok(())
}
```

### 建立连接

可以使用任何数据库连接类型并调用 `connect()` 建立单个连接。

```rust
use sqlx::Connection;
use sqlx_xugu::XuguConnection;

let conn = XuguConnection::connect("xugu://user:password@127.0.0.1:5138/SYSTEM").await?;
```

通常，您需要为应用程序创建一个连接池 (`sqlx::Pool`) 来调节它正在使用的服务器端连接数。

```rust
let pool = XuguPool::connect("xugu://user:password@127.0.0.1:5138/SYSTEM").await?;
```

### 更多请参考 `sqlx` 相关文档

* `docs.rs`: https://docs.rs/sqlx/latest/sqlx/ 
* `Github`: https://github.com/launchbadge/sqlx 
* `crates.io`: https://crates.io/crates/sqlx 


