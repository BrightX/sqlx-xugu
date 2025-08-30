mod interval;

use crate::arguments::XuguArgumentValue;
use crate::protocol::text::ColumnType;
use crate::{Xugu, XuguTypeInfo, XuguValueRef};
use bytes::{Buf, BufMut};
use chrono::{
    DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeDelta, TimeZone,
    Utc,
};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::borrow::Cow;

impl<Tz: TimeZone> Type<Xugu> for DateTime<Tz> {
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

impl<Tz: TimeZone> Encode<'_, Xugu> for DateTime<Tz> {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let micros = self.timestamp_micros();
        let offset_secs = self.offset().fix().local_minus_utc();
        let offset_hm = (offset_secs / 60) as i16;

        let mut buf = Vec::with_capacity(10);
        buf.put_i64(micros);
        buf.put_i16(offset_hm);
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for DateTime<Utc> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        <DateTime<FixedOffset> as Decode<Xugu>>::decode(value).map(DateTime::from)
    }
}

impl<'r> Decode<'r, Xugu> for DateTime<Local> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        <DateTime<FixedOffset> as Decode<Xugu>>::decode(value).map(DateTime::from)
    }
}

impl<'r> Decode<'r, Xugu> for DateTime<FixedOffset> {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        // 不带时区的日期时间
        if ty == ColumnType::DATETIME || ty == ColumnType::DATE {
            let native = <NaiveDateTime as Decode<Xugu>>::decode(value)?;
            let local = native.and_local_timezone(Local).unwrap();
            let tz = local.fixed_offset();

            return Ok(tz);
        }

        let mut buf = value.as_bytes()?;
        let micros = buf.get_i64();
        let tz_hm = buf.get_i16();
        let offset = FixedOffset::east_opt(tz_hm as i32 * 60).unwrap();

        let utc = DateTime::from_timestamp_micros(micros).unwrap();
        let tz = utc.with_timezone(&offset);

        Ok(tz)
    }
}

impl Type<Xugu> for NaiveTime {
    fn type_info() -> XuguTypeInfo {
        XuguTypeInfo::binary(ColumnType::TIME)
    }

    fn compatible(ty: &XuguTypeInfo) -> bool {
        matches!(ty.r#type, ColumnType::TIME | ColumnType::TIME_TZ)
    }
}

impl Encode<'_, Xugu> for NaiveTime {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let millis = (*self - NaiveTime::default()).num_milliseconds();

        let buf = (millis as i32).to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for NaiveTime {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        if ty == ColumnType::TIME_TZ {
            let mut buf = value.as_bytes()?;
            let millis = buf.get_i32();
            let tz_hm = buf.get_i16();

            let time = NaiveTime::MIN + (TimeDelta::milliseconds(millis as i64) + TimeDelta::minutes(tz_hm as i64));

            return Ok(time);
        }

        let millis = <i32 as Decode<Xugu>>::decode(value)?;
        let time = NaiveTime::MIN + TimeDelta::milliseconds(millis as i64);

        Ok(time)
    }
}

impl Type<Xugu> for NaiveDateTime {
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

impl Encode<'_, Xugu> for NaiveDateTime {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let micros: i64 = self.and_utc().timestamp_micros();

        let buf = micros.to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for NaiveDateTime {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        // 带时区的日期时间
        if ty == ColumnType::DATETIME_TZ {
            let datetime = <DateTime<FixedOffset> as Decode<Xugu>>::decode(value)?;

            return Ok(datetime.naive_local());
        }

        // 不带时间的日期
        if ty == ColumnType::DATE {
            let date = <NaiveDate as Decode<Xugu>>::decode(value)?;
            return Ok(date.and_time(NaiveTime::default()));
        }

        let millis = <i64 as Decode<Xugu>>::decode(value)?;
        let native = DateTime::from_timestamp_millis(millis).unwrap().naive_utc();

        Ok(native)
    }
}

impl Type<Xugu> for NaiveDate {
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

impl Encode<'_, Xugu> for NaiveDate {
    fn encode_by_ref(&self, args: &mut Vec<XuguArgumentValue>) -> Result<IsNull, BoxDynError> {
        let days: i64 = (*self - NaiveDate::default()).num_days();

        let buf = (days as i32).to_be_bytes().to_vec();
        args.push(XuguArgumentValue::Bin(Cow::Owned(buf)));

        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Xugu> for NaiveDate {
    fn decode(value: XuguValueRef<'r>) -> Result<Self, BoxDynError> {
        let ty = value.type_info.r#type;
        // 带时间的日期时间
        if ty == ColumnType::DATETIME_TZ || ty == ColumnType::DATETIME {
            let datetime = <NaiveDateTime as Decode<Xugu>>::decode(value)?;

            return Ok(datetime.date());
        }

        let days = <i32 as Decode<Xugu>>::decode(value)?;
        let date = NaiveDate::default() + TimeDelta::days(days as i64);

        Ok(date)
    }
}
