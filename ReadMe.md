# sqlx-xugu

[![github](https://img.shields.io/badge/Github-BrightX/sqlx--xugu-blue?logo=github)](https://github.com/BrightX/sqlx-xugu)
[![gitee](https://img.shields.io/badge/Gitee-BrightXu/sqlx--xugu-8da0cb?labelColor=C71D23&logo=gitee)](https://gitee.com/BrightXu/sqlx-xugu)
[![crate](https://img.shields.io/crates/v/sqlx-xugu.svg?logo=rust)](https://crates.io/crates/sqlx-xugu)
[![documentation](https://img.shields.io/badge/docs.rs-sqlx--xugu-66c2a5?labelColor=555555&logo=docs.rs)](https://docs.rs/sqlx-xugu)
[![minimum rustc 1.60](https://img.shields.io/badge/rustc-1.60+-red.svg?logo=rust)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
![License](https://img.shields.io/crates/l/sqlx-xugu)

> 基于 `sqlx` 的 `rust` 虚谷数据库驱动。
> 
> 支持虚谷数据库 `v11`/`v12`。

## Install

添加依赖，注意版本要跟 `sqlx` 保持一致

```toml
# Cargo.toml
sqlx = { version = "=0.8.6", features = ["runtime-tokio"] }
sqlx-xugu = { version = "0.8.6" }

```

#### Cargo Feature Flags

添加 feature, 注意要跟 `sqlx` 保持一致

```toml
# Cargo.toml
sqlx = { version = "=0.8.6", features = ["runtime-tokio", "chrono", "uuid", "rust_decimal", "bigdecimal", "time"] }
sqlx-xugu = { version = "0.8.6", features = ["chrono", "uuid", "rust_decimal", "bigdecimal", "time"] }
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

#[tokio::main] // Requires the `attributes` feature of `tokio`
// or #[async_std::main]
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

可以使用 `XuguConnection` 并调用 `connect()` 建立单个连接。

```rust
use sqlx::Connection;
use sqlx_xugu::XuguConnection;

let conn = XuguConnection::connect("xugu://user:password@127.0.0.1:5138/SYSTEM").await?;
```

通常，您需要为应用程序创建一个连接池 (`sqlx::Pool`) 来调节它正在使用的服务器端连接数。

```rust
let pool = XuguPool::connect("xugu://user:password@127.0.0.1:5138/SYSTEM").await?;

// 指定连接池大小和连接时间等参数
let pool = XuguPoolOptions::new()
    .max_connections(50)
    .acquire_timeout(std::time::Duration::from_secs(5))
    .idle_timeout(std::time::Duration::from_secs(300))
    .max_lifetime(std::time::Duration::from_secs(86400))
    .connect("xugu://user:password@127.0.0.1:5138/SYSTEM?ssl=ssl")
    .await?;
```

sqlx-xugu 驱动url格式，支持以下几种

```
1. xugu://ip:port/databaseName[?property=value[&property=value]]
  * 例：xugu://127.0.0.1:5138/SYSTEM?user=GUEST&password=GUEST&version=301&time_zone=GMT

2. xugu://user:passwd@ip:port/databaseName[?property=value[&property=value]]
  * 例：xugu://GUEST:GUEST@127.0.0.1:5138/SYSTEM?version=301&time_zone=GMT

```

#### 连接参数

| 参数名                       | 说明                                                                                                                         | 初始值                |
|:--------------------------|:---------------------------------------------------------------------------------------------------------------------------|:-------------------|
| database                  | 数据库名                                                                                                                       |                    |
| user                      | 用户名                                                                                                                        |                    |
| password                  | 用户密码                                                                                                                       |                    |
| version                   | 服务器版本                                                                                                                      | 301                |
| encryptor                 | 数据库解密密钥                                                                                                                    |                    |
| charset                   | 客户端字符集(**utf8**或**gbk**)                                                                                                   | utf8               |
| lob_ret                   | 大对象返回方式                                                                                                                    |                    |
| time_zone                 | 客户端时区                                                                                                                      |                    |
| iso_level                 | 事务隔离级别                                                                                                                     | READ COMMITTED读已提交 |
| lock_timeout              | 最大加锁等候时间                                                                                                                   |                    |
| auto_commit               | 是否自动提交                                                                                                                     | on                 |
| return_rowid              | 是否返回**rowid**                                                                                                              | false              |
| return_schema             | 查询**SQL**是否返回模式信息（此参数存在一个疑问）                                                                                               | on                 |
| identity_mode             | 数据库服务端自增长使用模式（**DEFAULT**：**default**自增,**NULL_AS_AUTO_INCREMENT**：**NULL**自增,**ZERO_AS_AUTO_INCREMENT**：**0**和**NULL**自增） |                    |
| keyword_filter            | 数据库连接配置连接上需要开放的关键字串，已逗号分隔，例如**TABLE,FUNCTION,CONSTANT**                                                                    |                    |
| disable_binlog            | 不记载**binlog**日志                                                                                                            |                    |
| current_schema            | 指定连接的模式名                                                                                                                   |                    |
| compatible_mode           | 适配其他数据库(**MySQL/ORACLE/PostgreSQL**)                                                                                       | NONE               |
| use_ssl                   | 是否开启传输数据加密保护 `on`: 启用加密，`off`: 禁用加密                                                                                        | off                |
| ssl                       | 同上 `ssl=ssl`: 启用加密，`ssl=nssl`: 禁用加密                                                                                        | nssl               |
| statement-cache-capacity  | 单个连接会话上的最大prepared语句数（max_prepare_num） 取值范围 `[100, 2097152]`，不要超过数据库设置的值 `show max_prepare_num;`            | 100                |

### 更多请参考 `sqlx` 相关文档

* `docs.rs`: https://docs.rs/sqlx/latest/sqlx/ 
* `Github`: https://github.com/launchbadge/sqlx 
* `crates.io`: https://crates.io/crates/sqlx 

## 联系方式

- **Bug 反馈**: [GitHub Issues](https://github.com/BrightX/sqlx-xugu/issues)
- **一般讨论**: [GitHub Discussions](https://github.com/BrightX/sqlx-xugu/discussions)
- **商务合作**: [BrightXu666@163.com](mailto:BrightXu666@163.com)

## 许可证

[MIT](https://opensource.org/license/mit) or [Apache 2.0](https://opensource.org/licenses/Apache-2.0)

### 免责声明

**sqlx-xugu** 跟 **成都虚谷伟业科技有限公司** 不构成任何知识产权归属关系。这个程序不含任何担保。

软件以“现状”提供，不提供任何明示或暗示的保证，包括但不限于可销性、特定用途适用性和非侵权等保证。
无论如何，作者或版权持有人对因软件或软件使用或其他交易产生的、涉及合同、侵权或其他诉讼的索赔、损害或其他责任均不承担责任。
