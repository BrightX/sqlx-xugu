use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::types::MICROS_PER_DAY_F;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use bytes::Buf;
use chrono::TimeDelta;
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;

impl Type<Xugu> for TimeDelta {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::INTERVAL_D2S)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::INTERVAL_D
                | ColumnType::INTERVAL_D2H
                | ColumnType::INTERVAL_D2M
                | ColumnType::INTERVAL_D2S
                | ColumnType::INTERVAL_H
                | ColumnType::INTERVAL_H2M
                | ColumnType::INTERVAL_H2S
                | ColumnType::INTERVAL_MI
                | ColumnType::INTERVAL_M2S
                | ColumnType::INTERVAL_S
                | ColumnType::NUMERIC
        )
    }
}

impl Encode<'_, Xugu> for TimeDelta {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let us: i64 = self.num_microseconds().unwrap_or_default();

        let buf = us.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for TimeDelta {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        // 数值型，单位：天
        if ty == ColumnType::NUMERIC {
            let d_str = value.as_str()?;
            let days: f64 = d_str.parse()?;
            let us_f = days * MICROS_PER_DAY_F;
            let us: i64 = us_f.round() as i64;
            let delta = TimeDelta::microseconds(us);

            return Ok(delta);
        }

        let mut buf = value.as_bytes()?;

        match ty {
            // 精确到天
            ColumnType::INTERVAL_D => {
                let day: i32 = buf.get_i32();
                let delta = TimeDelta::days(day as i64);
                Ok(delta)
            }
            // 精确到小时
            ColumnType::INTERVAL_D2H | ColumnType::INTERVAL_H => {
                let h: i32 = buf.get_i32();
                let delta = TimeDelta::hours(h as i64);
                Ok(delta)
            }
            // 精确到分钟
            ColumnType::INTERVAL_D2M | ColumnType::INTERVAL_H2M | ColumnType::INTERVAL_MI => {
                let min: i32 = buf.get_i32();
                let delta = TimeDelta::minutes(min as i64);
                Ok(delta)
            }
            // 精确到秒
            ColumnType::INTERVAL_D2S
            | ColumnType::INTERVAL_H2S
            | ColumnType::INTERVAL_M2S
            | ColumnType::INTERVAL_S => {
                let us: i64 = buf.get_i64();
                let delta = TimeDelta::microseconds(us);
                Ok(delta)
            }
            _ => Err(BoxDynError::from(
                "[E50044] Resultset: Required type conversion not allowed",
            )),
        }
    }
}
