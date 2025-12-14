mod interval;

use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};

use bytes::{Buf, BufMut};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;
use time::{Date, Duration, OffsetDateTime, PrimitiveDateTime, Time, UtcDateTime, UtcOffset};

impl Type<Xugu> for OffsetDateTime {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::DATETIME_TZ)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::DATETIME | ColumnType::DATE | ColumnType::DATETIME_TZ
        )
    }
}

impl Encode<'_, Xugu> for OffsetDateTime {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let unix_secs = self.unix_timestamp();
        let micro = self.microsecond();
        let micros = unix_secs * 1000_000 + micro as i64;
        let offset_hm = self.offset().whole_minutes();

        let mut buf = Vec::with_capacity(10);
        buf.put_i64(micros);
        buf.put_i16(offset_hm);
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for OffsetDateTime {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        // 不带时区的日期时间
        if ty == ColumnType::DATETIME || ty == ColumnType::DATE {
            let native = <PrimitiveDateTime as Decode<Xugu>>::decode(value)?;
            let local = OffsetDateTime::now_local().unwrap();
            let tz = local.replace_date_time(native);

            return Ok(tz);
        }

        let mut buf = value.as_bytes()?;
        let micros = buf.get_i64();
        let tz_hm = buf.get_i16();

        let nanos = micros as i128 * 1000;
        let offset = UtcOffset::from_whole_seconds(tz_hm as i32 * 60).unwrap();
        let utc = OffsetDateTime::from_unix_timestamp_nanos(nanos).unwrap();

        let tz = utc.to_offset(offset);

        Ok(tz)
    }
}

impl Type<Xugu> for PrimitiveDateTime {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::DATETIME)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::DATETIME | ColumnType::DATE | ColumnType::DATETIME_TZ
        )
    }
}

impl Encode<'_, Xugu> for PrimitiveDateTime {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        const UNIX_EPOCH: PrimitiveDateTime = UtcDateTime::UNIX_EPOCH.date().midnight();

        let micros: i64 = (*self - UNIX_EPOCH)
            .whole_microseconds()
            .try_into()
            .map_err(|_| {
                format!("value {self:?} would overflow binary encoding for Xugu DATETIME")
            })?;

        let buf = micros.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for PrimitiveDateTime {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        // 带时区的日期时间
        if ty == ColumnType::DATETIME_TZ {
            let datetime = <OffsetDateTime as Decode<Xugu>>::decode(value)?;
            let date = datetime.date();
            let time = datetime.time();

            return Ok(PrimitiveDateTime::new(date, time));
        }

        // 不带时间的日期
        if ty == ColumnType::DATE {
            let date = <Date as Decode<Xugu>>::decode(value)?;
            return Ok(date.midnight());
        }

        let millis = <i64 as Decode<Xugu>>::decode(value)?;
        let nanos = millis as i128 * 1000_000;
        let utc = UtcDateTime::from_unix_timestamp_nanos(nanos)?;

        Ok(PrimitiveDateTime::new(utc.date(), utc.time()))
    }
}

impl Type<Xugu> for Date {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::DATE)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(
            ty.r#type,
            ColumnType::DATE | ColumnType::DATETIME | ColumnType::DATETIME_TZ
        )
    }
}

impl Encode<'_, Xugu> for Date {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        const UNIX_EPOCH: Date = UtcDateTime::UNIX_EPOCH.date();

        let days: i64 = (*self - UNIX_EPOCH).whole_days();

        let buf = (days as i32).to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for Date {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        // 带时间的日期时间
        if ty == ColumnType::DATETIME_TZ || ty == ColumnType::DATETIME {
            let datetime = <PrimitiveDateTime as Decode<Xugu>>::decode(value)?;

            return Ok(datetime.date());
        }
        const UNIX_EPOCH: Date = UtcDateTime::UNIX_EPOCH.date();

        let days = <i32 as Decode<Xugu>>::decode(value)?;
        let date = UNIX_EPOCH + Duration::days(days as i64);

        Ok(date)
    }
}

impl Type<Xugu> for Time {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::TIME)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(ty.r#type, ColumnType::TIME | ColumnType::TIME_TZ)
    }
}

impl Encode<'_, Xugu> for Time {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let millis = (*self - Time::MIDNIGHT).whole_milliseconds();

        let buf = (millis as i32).to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for Time {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        if ty == ColumnType::TIME_TZ {
            let mut buf = value.as_bytes()?;
            let millis = buf.get_i32();
            let tz_hm = buf.get_i16();

            let time = Time::MIDNIGHT
                + (Duration::milliseconds(millis as i64) + Duration::minutes(tz_hm as i64));

            return Ok(time);
        }

        let millis = <i32 as Decode<Xugu>>::decode(value)?;
        let time = Time::MIDNIGHT + Duration::milliseconds(millis as i64);

        Ok(time)
    }
}
