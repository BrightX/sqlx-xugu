mod bool;
mod bytes;
mod float;
mod int;
mod str;
mod text;
mod uint;

/// The number of microseconds per in days.
#[allow(dead_code)]
const MICROS_PER_DAY: i64 = 86_400_000_000;
/// The number of microseconds per in days.
const MICROS_PER_DAY_F: f64 = 86_400_000_000.0;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "bigdecimal")]
mod bigdecimal;

#[cfg(feature = "rust_decimal")]
mod rust_decimal;

#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "uuid")]
mod uuid;
