use crate::io::{AsyncStreamExt, StreamDecode};
use crate::protocol::ServerContext;
use bitflags::bitflags;
use sqlx_core::bytes::Bytes;
use sqlx_core::{err_protocol, Error};

bitflags! {
    #[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct ColumnFlags: i32 {
        /// 字段是实际的表的字段(否则,表示字段实际上是表达式)
        const BASE_TAB = 1;
        /// 字段具有非空约束
        const NOT_NULL = 2;
        /// 字段是主键
        const IS_PRIMARY = 4;
        /// 字段是序列值
        const IS_SERIAL = 8;
        /// Field is a timestamp.
        const IS_TIMESTAMP = 16;
        /// 字段是大对象类型
        const IS_LOB = 32;
        /// 字段是唯一值类型
        const IS_UNIQUE = 64;
        /// Field is rowid.
        const IS_ROWID = 128;
        /// 字段不应输出
        const IS_DUMMY = 256;
        /// 字段应该被隐藏
        const IS_HIDE = 512;
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
#[repr(i32)]
pub enum ColumnType {
    NONE = 0x00,
    NULL = 0x01,
    /// 布尔类型；1 byte
    BOOLEAN = 0x02,
    /// 微整型数据类型；1字节
    TINYINT = 0x03,
    /// 短整型数据类型；2字节
    SMALLINT = 0x04,
    /// 整型数据类型；4字节
    INTEGER = 0x05,
    /// 长整型数据类型；8字节
    BIGINT = 0x06,
    /// 固定精度的数据类型；
    NUMERIC = 0x07,
    /// 单精度型数据类型；4字节
    FLOAT = 0x08,
    /// 双精度型数据类型；8字节
    DOUBLE = 0x09,
    /// 日期数据类型；
    DATE = 0x0a,
    /// 不含年月日的时间数据类型；
    TIME = 0x0b,
    TIME_TZ = 0x0c,
    /// 包含年月日的时间数据类型；
    DATETIME = 0x0d,
    DATETIME_TZ = 0x0e,
    /// 只有年的时间间隔
    INTERVAL_Y = 0x0f,
    INTERVAL_Y2M = 0x10,
    /// 只有月的时间间隔
    INTERVAL_M = 0x11,
    /// 只有日的时间间隔
    INTERVAL_D = 0x12,
    INTERVAL_D2H = 0x13,
    /// 只有时的时间间隔
    INTERVAL_H = 0x14,
    INTERVAL_D2M = 0x15,
    INTERVAL_H2M = 0x16,
    /// 只有分钟的时间间隔
    INTERVAL_MI = 0x17,
    INTERVAL_D2S = 0x18,
    INTERVAL_H2S = 0x19,
    INTERVAL_M2S = 0x1a,
    /// 只有秒的时间间隔
    INTERVAL_S = 0x1b,
    ROWVERSION = 0x1c,
    /// 唯一值标志类型
    GUID = 0x1d,
    /// 定长的字符数据类型；
    CHAR = 0x1e,
    NCHAR = 31,
    CLOB = 0x20,
    /// 普通二进制类型；
    BINARY = 0x21,
    /// 大二进制类型
    BLOB = 0x22,

    GEOMETRY_OLD = 35,
    POINT_OLD = 36,
    BOX_OLD = 37,
    LINE_OLD = 38,
    POLYGON_OLD = 39,
    BLOB_I = 40,
    BLOB_S = 41,
    BLOB_M = 42,
    BLOB_OM = 43,
    STREAM = 44,

    ROWID = 45,
    SIBLING = 46,
    JSON = 47,

    /// 点类型
    POINT = 48,
    /// 有限线段类型
    LSEG = 49,
    /// 折线类型
    LINE = 50,
    /// 边框类型
    BOX = 51,
    /// 路径类型
    PATH = 52,
    /// 多边形类型
    POLYGON = 53,
    /// 圆类型
    CIRCLE = 54,
    /// 通用空间类型
    GEOMETRY = 55,
    GEOGRAPHY = 56,
    BOX2D = 57,
    BOX3D = 58,
    SPHEROID = 59,
    RASTER = 60,
    ST_END = 61,

    BIT = 62,
    VARBIT = 63,
    XML = 64,
    ARRAY = 65,
    BLADE_BEGIN = 101,
    BLADE_END = 1000,

    /// 自定义数据类型的 OBJECT
    OBJECT = 1001,
    REFROW = 1002,
    /// 自定义数据类型的 RECORD（记录类型）
    RECORD = 1003,
    /// 自定义数据类型的 VARRAY
    VARRAY = 1004,
    /// 自定义数据类型的 TABLE
    TABLE = 1005,
    /// Idxby表
    ITABLE = 1006,

    /// 游标（引用记录）类型
    CURSOR = 1007,

    /// REF_CURSOR类型
    REFCUR = 1008,

    /// 引用行类型
    ROWTYPE = 1009,

    /// 引用列类型
    COLTYPE = 1010,

    /// CUR_REC类型
    CUR_REC = 1011,

    /// PARAM类型
    PARAM = 1012,

    /// MAX_ALL
    MAX_ALL = 1013,

    /// 数组类型
    ARRAY_NONE = 10000,
    ARRAY_NULL = 10001,
    ARRAY_BOOLEAN = 10002,
    ARRAY_TINYINT = 10003,
    ARRAY_SMALLINT = 10004,
    ARRAY_INTEGER = 10005,
    ARRAY_BIGINT = 10006,
    ARRAY_NUMERIC = 10007,
    ARRAY_FLOAT = 10008,
    ARRAY_DOUBLE = 10009,
    ARRAY_DATE = 10010,
    ARRAY_TIME = 10011,
    ARRAY_TIME_TZ = 10012,
    ARRAY_DATETIME = 10013,
    ARRAY_DATETIME_TZ = 10014,
    ARRAY_INTERVAL_Y = 10015,
    ARRAY_INTERVAL_Y2M = 10016,
    ARRAY_INTERVAL_M = 10017,
    ARRAY_INTERVAL_D = 10018,
    ARRAY_INTERVAL_D2H = 10019,
    ARRAY_INTERVAL_H = 10020,
    ARRAY_INTERVAL_D2M = 10021,
    ARRAY_INTERVAL_H2M = 10022,
    ARRAY_INTERVAL_MI = 10023,
    ARRAY_INTERVAL_D2S = 10024,
    ARRAY_INTERVAL_H2S = 10025,
    ARRAY_INTERVAL_M2S = 10026,
    ARRAY_INTERVAL_S = 10027,
    ARRAY_ROWVERSION = 10028,
    ARRAY_GUID = 10029,
    ARRAY_CHAR = 10030,
    ARRAY_NCHAR = 10031,
    ARRAY_CLOB = 10032,
    ARRAY_BINARY = 10033,
    ARRAY_BLOB = 10034,
    ARRAY_GEOMETRY_OLD = 10035,
    ARRAY_POINT_OLD = 10036,
    ARRAY_BOX_OLD = 10037,
    ARRAY_LINE_OLD = 10038,
    ARRAY_POLYGON_OLD = 10039,
    ARRAY_BLOB_I = 10040,
    ARRAY_BLOB_S = 10041,
    ARRAY_BLOB_M = 10042,
    ARRAY_BLOB_OM = 10043,
    ARRAY_STREAM = 10044,
    ARRAY_ROWID = 10045,
    ARRAY_SIBLING = 10046,
    ARRAY_JSON = 10047,
    ARRAY_POINT = 10048,
    ARRAY_LSEG = 10049,
    ARRAY_LINE = 10050,
    ARRAY_BOX = 10051,
    ARRAY_PATH = 10052,
    ARRAY_POLYGON = 10053,
    ARRAY_CIRCLE = 10054,
    ARRAY_GEOMETRY = 10055,
    ARRAY_GEOGRAPHY = 10056,
    ARRAY_BOX2D = 10057,
    ARRAY_BOX3D = 10058,
    ARRAY_SPHEROID = 10059,
    ARRAY_RASTER = 10060,
    ARRAY_ST_END = 10061,
    ARRAY_BIT = 10062,
    ARRAY_VARBIT = 10063,
    ARRAY_MAX_SYS = 10064,
    ARRAY_BLADE_BEGIN = 10101,
    ARRAY_BLADE_END = 11000,
    ARRAY_OBJECT = 11001,
    ARRAY_REFROW = 11002,
    ARRAY_RECORD = 11003,
    ARRAY_VARRAY = 11004,
    ARRAY_TABLE = 11005,
    ARRAY_ITABLE = 11006,
    ARRAY_CURSOR = 11007,
    ARRAY_REFCUR = 11008,
    ARRAY_ROWTYPE = 11009,
    ARRAY_COLTYPE = 11010,
    ARRAY_CUR_REC = 11011,
    ARRAY_PARAM = 11012,
    ARRAY_MAX_ALL = 11013,
}

impl ColumnType {
    pub(crate) fn name(self) -> &'static str {
        match self {
            ColumnType::NONE => "none",
            ColumnType::NULL => "null",
            ColumnType::BOOLEAN => "BOOLEAN",
            ColumnType::TINYINT => "TINYINT",
            ColumnType::SMALLINT => "SMALLINT",
            ColumnType::INTEGER => "INTEGER",
            ColumnType::BIGINT => "BIGINT",
            ColumnType::NUMERIC => "NUMERIC",
            ColumnType::FLOAT => "FLOAT",
            ColumnType::DOUBLE => "DOUBLE",
            ColumnType::DATE => "DATE",
            ColumnType::TIME => "TIME",
            ColumnType::TIME_TZ => "TIME WITH TIME ZONE",
            ColumnType::DATETIME => "DATETIME",
            ColumnType::DATETIME_TZ => "DATETIME WITH TIME ZONE",
            ColumnType::INTERVAL_Y => "INTERVAL YEAR",
            ColumnType::INTERVAL_Y2M => "INTERVAL YEAR TO MONTH",
            ColumnType::INTERVAL_M => "INTERVAL MONTH",
            ColumnType::INTERVAL_D => "INTERVAL DAY",
            ColumnType::INTERVAL_D2H => "INTERVAL DAY TO HOUR",
            ColumnType::INTERVAL_H => "INTERVAL HOUR",
            ColumnType::INTERVAL_D2M => "INTERVAL DAY TO MINUTE",
            ColumnType::INTERVAL_H2M => "INTERVAL HOUR TO MINUTE",
            ColumnType::INTERVAL_MI => "INTERVAL MINUTE",
            ColumnType::INTERVAL_D2S => "INTERVAL DAY TO SECOND",
            ColumnType::INTERVAL_H2S => "INTERVAL HOUR TO SECOND",
            ColumnType::INTERVAL_M2S => "INTERVAL MINUTE TO SECOND",
            ColumnType::INTERVAL_S => "INTERVAL SECOND",
            ColumnType::ROWVERSION => "ROWVERSION",
            ColumnType::GUID => "GUID",
            ColumnType::CHAR => "CHAR",
            ColumnType::NCHAR => "NCHAR",
            ColumnType::CLOB => "CLOB",
            ColumnType::BINARY => "BINARY",
            ColumnType::BLOB => "BLOB",

            ColumnType::GEOMETRY_OLD => "GEOMETRY_OLD",
            ColumnType::POINT_OLD => "POINT_OLD",
            ColumnType::BOX_OLD => "BOX_OLD",
            ColumnType::LINE_OLD => "LINE_OLD",
            ColumnType::POLYGON_OLD => "POLYGON_OLD",
            ColumnType::BLOB_I => "BLOB_I",
            ColumnType::BLOB_S => "BLOB_S",
            ColumnType::BLOB_M => "BLOB_M",
            ColumnType::BLOB_OM => "BLOB_OM",
            ColumnType::STREAM => "STREAM",

            ColumnType::ROWID => "ROWID",
            ColumnType::SIBLING => "SIBLING",
            ColumnType::JSON => "JSON",

            ColumnType::POINT => "POINT",
            ColumnType::LSEG => "LSEG",
            ColumnType::LINE => "LINE",
            ColumnType::BOX => "BOX",
            ColumnType::PATH => "PATH",
            ColumnType::POLYGON => "POLYGON",
            ColumnType::CIRCLE => "CIRCLE",
            ColumnType::GEOMETRY => "GEOMETRY",
            ColumnType::GEOGRAPHY => "GEOGRAPHY",
            ColumnType::BOX2D => "BOX2D",
            ColumnType::BOX3D => "BOX3D",
            ColumnType::SPHEROID => "SPHEROID",
            ColumnType::RASTER => "RASTER",
            ColumnType::ST_END => "ST_END",
            ColumnType::BIT => "BIT",
            ColumnType::VARBIT => "VARBIT",

            ColumnType::XML => "XML",
            ColumnType::ARRAY => "ARRAY",
            ColumnType::BLADE_BEGIN => "BLADE_BEGIN",
            ColumnType::BLADE_END => "BLADE_END",

            ColumnType::OBJECT => "OBJECT",
            ColumnType::REFROW => "REFROW",
            ColumnType::RECORD => "RECORD",
            ColumnType::VARRAY => "VARRAY",
            ColumnType::TABLE => "UDT TABLE",
            ColumnType::ITABLE => "index-by table",

            ColumnType::CURSOR => "CURSOR",

            ColumnType::REFCUR => "SYS_REFCURSOR",

            ColumnType::ROWTYPE => "ROWTYPE",

            ColumnType::COLTYPE => "COLTYPE",

            ColumnType::CUR_REC => "CUR_REC",

            ColumnType::PARAM => "PARAM",

            ColumnType::MAX_ALL => "MAX_ALL",

            ColumnType::ARRAY_NONE => "ARRAY_NONE",
            ColumnType::ARRAY_NULL => "ARRAY_NULL",
            ColumnType::ARRAY_BOOLEAN => "ARRAY_BOOLEAN",
            ColumnType::ARRAY_TINYINT => "ARRAY_TINYINT",
            ColumnType::ARRAY_SMALLINT => "ARRAY_SMALLINT",
            ColumnType::ARRAY_INTEGER => "ARRAY_INTEGER",
            ColumnType::ARRAY_BIGINT => "ARRAY_BIGINT",
            ColumnType::ARRAY_NUMERIC => "ARRAY_NUMERIC",
            ColumnType::ARRAY_FLOAT => "ARRAY_FLOAT",
            ColumnType::ARRAY_DOUBLE => "ARRAY_DOUBLE",
            ColumnType::ARRAY_DATE => "ARRAY_DATE",
            ColumnType::ARRAY_TIME => "ARRAY_TIME",
            ColumnType::ARRAY_TIME_TZ => "ARRAY_TIME_TZ",
            ColumnType::ARRAY_DATETIME => "ARRAY_DATETIME",
            ColumnType::ARRAY_DATETIME_TZ => "ARRAY_DATETIME_TZ",
            ColumnType::ARRAY_INTERVAL_Y => "ARRAY_INTERVAL_Y",
            ColumnType::ARRAY_INTERVAL_Y2M => "ARRAY_INTERVAL_Y2M",
            ColumnType::ARRAY_INTERVAL_M => "ARRAY_INTERVAL_M",
            ColumnType::ARRAY_INTERVAL_D => "ARRAY_INTERVAL_D",
            ColumnType::ARRAY_INTERVAL_D2H => "ARRAY_INTERVAL_D2H",
            ColumnType::ARRAY_INTERVAL_H => "ARRAY_INTERVAL_H",
            ColumnType::ARRAY_INTERVAL_D2M => "ARRAY_INTERVAL_D2M",
            ColumnType::ARRAY_INTERVAL_H2M => "ARRAY_INTERVAL_H2M",
            ColumnType::ARRAY_INTERVAL_MI => "ARRAY_INTERVAL_MI",
            ColumnType::ARRAY_INTERVAL_D2S => "ARRAY_INTERVAL_D2S",
            ColumnType::ARRAY_INTERVAL_H2S => "ARRAY_INTERVAL_H2S",
            ColumnType::ARRAY_INTERVAL_M2S => "ARRAY_INTERVAL_M2S",
            ColumnType::ARRAY_INTERVAL_S => "ARRAY_INTERVAL_S",
            ColumnType::ARRAY_ROWVERSION => "ARRAY_ROWVERSION",
            ColumnType::ARRAY_GUID => "ARRAY_GUID",
            ColumnType::ARRAY_CHAR => "ARRAY_CHAR",
            ColumnType::ARRAY_NCHAR => "ARRAY_NCHAR",
            ColumnType::ARRAY_CLOB => "ARRAY_CLOB",
            ColumnType::ARRAY_BINARY => "ARRAY_BINARY",
            ColumnType::ARRAY_BLOB => "ARRAY_BLOB",
            ColumnType::ARRAY_GEOMETRY_OLD => "ARRAY_GEOMETRY_OLD",
            ColumnType::ARRAY_POINT_OLD => "ARRAY_POINT_OLD",
            ColumnType::ARRAY_BOX_OLD => "ARRAY_BOX_OLD",
            ColumnType::ARRAY_LINE_OLD => "ARRAY_LINE_OLD",
            ColumnType::ARRAY_POLYGON_OLD => "ARRAY_POLYGON_OLD",
            ColumnType::ARRAY_BLOB_I => "ARRAY_BLOB_I",
            ColumnType::ARRAY_BLOB_S => "ARRAY_BLOB_S",
            ColumnType::ARRAY_BLOB_M => "ARRAY_BLOB_M",
            ColumnType::ARRAY_BLOB_OM => "ARRAY_BLOB_OM",
            ColumnType::ARRAY_STREAM => "ARRAY_STREAM",
            ColumnType::ARRAY_ROWID => "ARRAY_ROWID",
            ColumnType::ARRAY_SIBLING => "ARRAY_SIBLING",
            ColumnType::ARRAY_JSON => "ARRAY_JSON",
            ColumnType::ARRAY_POINT => "ARRAY_POINT",
            ColumnType::ARRAY_LSEG => "ARRAY_LSEG",
            ColumnType::ARRAY_LINE => "ARRAY_LINE",
            ColumnType::ARRAY_BOX => "ARRAY_BOX",
            ColumnType::ARRAY_PATH => "ARRAY_PATH",
            ColumnType::ARRAY_POLYGON => "ARRAY_POLYGON",
            ColumnType::ARRAY_CIRCLE => "ARRAY_CIRCLE",
            ColumnType::ARRAY_GEOMETRY => "ARRAY_GEOMETRY",
            ColumnType::ARRAY_GEOGRAPHY => "ARRAY_GEOGRAPHY",
            ColumnType::ARRAY_BOX2D => "ARRAY_BOX2D",
            ColumnType::ARRAY_BOX3D => "ARRAY_BOX3D",
            ColumnType::ARRAY_SPHEROID => "ARRAY_SPHEROID",
            ColumnType::ARRAY_RASTER => "ARRAY_RASTER",
            ColumnType::ARRAY_ST_END => "ARRAY_ST_END",
            ColumnType::ARRAY_BIT => "ARRAY_BIT",
            ColumnType::ARRAY_VARBIT => "ARRAY_VARBIT",
            ColumnType::ARRAY_MAX_SYS => "ARRAY_MAX_SYS",
            ColumnType::ARRAY_BLADE_BEGIN => "ARRAY_BLADE_BEGIN",
            ColumnType::ARRAY_BLADE_END => "ARRAY_BLADE_END",
            ColumnType::ARRAY_OBJECT => "ARRAY_OBJECT",
            ColumnType::ARRAY_REFROW => "ARRAY_REFROW",
            ColumnType::ARRAY_RECORD => "ARRAY_RECORD",
            ColumnType::ARRAY_VARRAY => "ARRAY_VARRAY",
            ColumnType::ARRAY_TABLE => "ARRAY_TABLE",
            ColumnType::ARRAY_ITABLE => "ARRAY_ITABLE",
            ColumnType::ARRAY_CURSOR => "ARRAY_CURSOR",
            ColumnType::ARRAY_REFCUR => "ARRAY_REFCUR",
            ColumnType::ARRAY_ROWTYPE => "ARRAY_ROWTYPE",
            ColumnType::ARRAY_COLTYPE => "ARRAY_COLTYPE",
            ColumnType::ARRAY_CUR_REC => "ARRAY_CUR_REC",
            ColumnType::ARRAY_PARAM => "ARRAY_PARAM",
            ColumnType::ARRAY_MAX_ALL => "ARRAY_MAX_ALL",
        }
    }

    pub(crate) fn try_from_i32(id: i32) -> Result<Self, Error> {
        Ok(match id {
            0x00 => ColumnType::NONE,
            0x01 => ColumnType::NULL,
            0x02 => ColumnType::BOOLEAN,
            0x03 => ColumnType::TINYINT,
            0x04 => ColumnType::SMALLINT,
            0x05 => ColumnType::INTEGER,
            0x06 => ColumnType::BIGINT,
            0x07 => ColumnType::NUMERIC,
            0x08 => ColumnType::FLOAT,
            0x09 => ColumnType::DOUBLE,
            0x0a => ColumnType::DATE,
            0x0b => ColumnType::TIME,
            0x0c => ColumnType::TIME_TZ,
            0x0d => ColumnType::DATETIME,
            0x0e => ColumnType::DATETIME_TZ,
            0x0f => ColumnType::INTERVAL_Y,
            0x10 => ColumnType::INTERVAL_Y2M,
            0x11 => ColumnType::INTERVAL_M,
            0x12 => ColumnType::INTERVAL_D,
            0x13 => ColumnType::INTERVAL_D2H,
            0x14 => ColumnType::INTERVAL_H,
            0x15 => ColumnType::INTERVAL_D2M,
            0x16 => ColumnType::INTERVAL_H2M,
            0x17 => ColumnType::INTERVAL_MI,
            0x18 => ColumnType::INTERVAL_D2S,
            0x19 => ColumnType::INTERVAL_H2S,
            0x1a => ColumnType::INTERVAL_M2S,
            0x1b => ColumnType::INTERVAL_S,
            0x1c => ColumnType::ROWVERSION,
            0x1d => ColumnType::GUID,
            0x1e => ColumnType::CHAR,
            31 => ColumnType::NCHAR,
            0x20 => ColumnType::CLOB,
            0x21 => ColumnType::BINARY,
            0x22 => ColumnType::BLOB,

            35 => ColumnType::GEOMETRY_OLD,
            36 => ColumnType::POINT_OLD,
            37 => ColumnType::BOX_OLD,
            38 => ColumnType::LINE_OLD,
            39 => ColumnType::POLYGON_OLD,
            40 => ColumnType::BLOB_I,
            41 => ColumnType::BLOB_S,
            42 => ColumnType::BLOB_M,
            43 => ColumnType::BLOB_OM,
            44 => ColumnType::STREAM,

            45 => ColumnType::ROWID,
            46 => ColumnType::SIBLING,
            47 => ColumnType::JSON,

            48 => ColumnType::POINT,
            49 => ColumnType::LSEG,
            50 => ColumnType::LINE,
            51 => ColumnType::BOX,
            52 => ColumnType::PATH,
            53 => ColumnType::POLYGON,
            54 => ColumnType::CIRCLE,
            55 => ColumnType::GEOMETRY,
            56 => ColumnType::GEOGRAPHY,
            57 => ColumnType::BOX2D,
            58 => ColumnType::BOX3D,
            59 => ColumnType::SPHEROID,
            60 => ColumnType::RASTER,
            61 => ColumnType::ST_END,

            62 => ColumnType::BIT,
            63 => ColumnType::VARBIT,
            64 => ColumnType::XML,
            65 => ColumnType::ARRAY,
            101 => ColumnType::BLADE_BEGIN,
            1000 => ColumnType::BLADE_END,

            1001 => ColumnType::OBJECT,
            1002 => ColumnType::REFROW,
            1003 => ColumnType::RECORD,
            1004 => ColumnType::VARRAY,
            1005 => ColumnType::TABLE,
            1006 => ColumnType::ITABLE,

            1007 => ColumnType::CURSOR,

            1008 => ColumnType::REFCUR,

            1009 => ColumnType::ROWTYPE,

            1010 => ColumnType::COLTYPE,

            1011 => ColumnType::CUR_REC,

            1012 => ColumnType::PARAM,

            1013 => ColumnType::MAX_ALL,

            10000 => ColumnType::ARRAY_NONE,
            10001 => ColumnType::ARRAY_NULL,
            10002 => ColumnType::ARRAY_BOOLEAN,
            10003 => ColumnType::ARRAY_TINYINT,
            10004 => ColumnType::ARRAY_SMALLINT,
            10005 => ColumnType::ARRAY_INTEGER,
            10006 => ColumnType::ARRAY_BIGINT,
            10007 => ColumnType::ARRAY_NUMERIC,
            10008 => ColumnType::ARRAY_FLOAT,
            10009 => ColumnType::ARRAY_DOUBLE,
            10010 => ColumnType::ARRAY_DATE,
            10011 => ColumnType::ARRAY_TIME,
            10012 => ColumnType::ARRAY_TIME_TZ,
            10013 => ColumnType::ARRAY_DATETIME,
            10014 => ColumnType::ARRAY_DATETIME_TZ,
            10015 => ColumnType::ARRAY_INTERVAL_Y,
            10016 => ColumnType::ARRAY_INTERVAL_Y2M,
            10017 => ColumnType::ARRAY_INTERVAL_M,
            10018 => ColumnType::ARRAY_INTERVAL_D,
            10019 => ColumnType::ARRAY_INTERVAL_D2H,
            10020 => ColumnType::ARRAY_INTERVAL_H,
            10021 => ColumnType::ARRAY_INTERVAL_D2M,
            10022 => ColumnType::ARRAY_INTERVAL_H2M,
            10023 => ColumnType::ARRAY_INTERVAL_MI,
            10024 => ColumnType::ARRAY_INTERVAL_D2S,
            10025 => ColumnType::ARRAY_INTERVAL_H2S,
            10026 => ColumnType::ARRAY_INTERVAL_M2S,
            10027 => ColumnType::ARRAY_INTERVAL_S,
            10028 => ColumnType::ARRAY_ROWVERSION,
            10029 => ColumnType::ARRAY_GUID,
            10030 => ColumnType::ARRAY_CHAR,
            10031 => ColumnType::ARRAY_NCHAR,
            10032 => ColumnType::ARRAY_CLOB,
            10033 => ColumnType::ARRAY_BINARY,
            10034 => ColumnType::ARRAY_BLOB,
            10035 => ColumnType::ARRAY_GEOMETRY_OLD,
            10036 => ColumnType::ARRAY_POINT_OLD,
            10037 => ColumnType::ARRAY_BOX_OLD,
            10038 => ColumnType::ARRAY_LINE_OLD,
            10039 => ColumnType::ARRAY_POLYGON_OLD,
            10040 => ColumnType::ARRAY_BLOB_I,
            10041 => ColumnType::ARRAY_BLOB_S,
            10042 => ColumnType::ARRAY_BLOB_M,
            10043 => ColumnType::ARRAY_BLOB_OM,
            10044 => ColumnType::ARRAY_STREAM,
            10045 => ColumnType::ARRAY_ROWID,
            10046 => ColumnType::ARRAY_SIBLING,
            10047 => ColumnType::ARRAY_JSON,
            10048 => ColumnType::ARRAY_POINT,
            10049 => ColumnType::ARRAY_LSEG,
            10050 => ColumnType::ARRAY_LINE,
            10051 => ColumnType::ARRAY_BOX,
            10052 => ColumnType::ARRAY_PATH,
            10053 => ColumnType::ARRAY_POLYGON,
            10054 => ColumnType::ARRAY_CIRCLE,
            10055 => ColumnType::ARRAY_GEOMETRY,
            10056 => ColumnType::ARRAY_GEOGRAPHY,
            10057 => ColumnType::ARRAY_BOX2D,
            10058 => ColumnType::ARRAY_BOX3D,
            10059 => ColumnType::ARRAY_SPHEROID,
            10060 => ColumnType::ARRAY_RASTER,
            10061 => ColumnType::ARRAY_ST_END,
            10062 => ColumnType::ARRAY_BIT,
            10063 => ColumnType::ARRAY_VARBIT,
            10064 => ColumnType::ARRAY_MAX_SYS,
            10101 => ColumnType::ARRAY_BLADE_BEGIN,
            11000 => ColumnType::ARRAY_BLADE_END,
            11001 => ColumnType::ARRAY_OBJECT,
            11002 => ColumnType::ARRAY_REFROW,
            11003 => ColumnType::ARRAY_RECORD,
            11004 => ColumnType::ARRAY_VARRAY,
            11005 => ColumnType::ARRAY_TABLE,
            11006 => ColumnType::ARRAY_ITABLE,
            11007 => ColumnType::ARRAY_CURSOR,
            11008 => ColumnType::ARRAY_REFCUR,
            11009 => ColumnType::ARRAY_ROWTYPE,
            11010 => ColumnType::ARRAY_COLTYPE,
            11011 => ColumnType::ARRAY_CUR_REC,
            11012 => ColumnType::ARRAY_PARAM,
            11013 => ColumnType::ARRAY_MAX_ALL,

            _ => {
                return Err(err_protocol!("unknown column type 0x{:02x}", id));
            }
        })
    }
}

#[allow(non_snake_case)]
#[derive(Debug)]
pub(crate) struct ColumnDefinition {
    /// 表的模式名
    #[allow(unused)]
    schema: Bytes,
    /// 数据库名称
    // #[allow(unused)]
    // database: Bytes,
    /// 表的名称
    #[allow(unused)]
    table: Bytes,
    /// 列的标签
    alias: Bytes,
    /// 列的名称
    name: Bytes,
    pub(crate) r#type: ColumnType,
    pub(crate) flags: ColumnFlags,
    /// 列的精度
    #[allow(unused)]
    precision: i32,
    /// 列的标度
    #[allow(unused)]
    scale: i32,
}

fn split_bytes_into_ranges(bytes: &Bytes, delimiter: u8) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = 0;

    for (index, &byte) in bytes.iter().enumerate() {
        if byte == delimiter {
            // 添加当前范围（不包括分隔符）
            ranges.push(start..index);
            // 下一个范围从分隔符之后开始
            start = index + 1;
        }
    }

    // 添加最后一个范围
    ranges.push(start..bytes.len());

    ranges
}
fn replace_0x01_to_dot(bytes: Bytes) -> Bytes {
    if bytes.is_empty() {
        return bytes;
    }
    let mut vec = bytes.to_vec();
    for byte in &mut vec {
        if *byte == 0x01 {
            *byte = b'.';
        }
    }
    vec.into()
}

impl ColumnDefinition {
    // NOTE: strings in-protocol are transmitted according to the client character set
    //       as this is UTF-8, all these strings should be UTF-8

    pub(crate) fn name(&self) -> Result<&str, Error> {
        std::str::from_utf8(&self.name).map_err(Error::protocol)
    }

    pub(crate) fn alias(&self) -> Result<&str, Error> {
        std::str::from_utf8(&self.alias).map_err(Error::protocol)
    }
}

impl StreamDecode<ServerContext> for ColumnDefinition {
    async fn decode_with<S: AsyncStreamExt>(
        stream: &mut S,
        cnt: ServerContext,
    ) -> Result<Self, Error> {
        let schema;
        let mut table;
        let mut name;
        let mut alias;

        if cnt.support_401() {
            //模式名
            let len = stream.read_u16().await?;
            schema = stream.read_bytes(len as usize).await?;
            //表名
            let len = stream.read_u16().await?;
            table = stream.read_bytes(len as usize).await?;
            //列名
            let len = stream.read_u16().await?;
            name = stream.read_bytes(len as usize).await?;
            //别名
            let len = stream.read_u16().await?;
            if len > 0 {
                alias = stream.read_bytes(len as usize).await?;
            } else {
                alias = name.clone();
            }
        } else {
            let len = stream.read_i32().await?;
            // NAME1
            // NAME1%alias
            // table.NAME1%alias
            // Schema.table.NAME1%alias
            let mut total_name = stream.read_bytes(len as usize).await?;

            //别名
            if let Some(pos) = total_name.iter().rposition(|&x| x == b'%') {
                let mut s = total_name.split_off(pos);
                alias = s.split_off(1);
            } else {
                alias = Bytes::new();
            };

            let ranges = split_bytes_into_ranges(&total_name, b'.');

            if ranges.len() >= 3 {
                schema = total_name.slice(ranges[0].clone());
                table = total_name.slice(ranges[1].clone());
                name = total_name.slice(ranges[2].clone());
            } else if ranges.len() == 2 {
                schema = Bytes::new();
                table = total_name.slice(ranges[0].clone());
                name = total_name.slice(ranges[1].clone());
            } else if ranges.len() == 1 {
                schema = Bytes::new();
                table = Bytes::new();
                name = total_name.slice(ranges[0].clone());
            } else {
                schema = Bytes::new();
                table = Bytes::new();
                name = total_name;
            }

            table = replace_0x01_to_dot(table);
            name = replace_0x01_to_dot(name);
            alias = replace_0x01_to_dot(alias);
        }

        let type_id = stream.read_i32().await?;
        let precision_scale = stream.read_i32().await?;
        let flags = stream.read_i32().await?;

        let r#type = ColumnType::try_from_i32(type_id)?;
        let precision;
        let scale;
        if r#type == ColumnType::NUMERIC {
            precision = precision_scale >> 16;
            scale = precision_scale & 0x0000ffff;
        } else {
            precision = precision_scale;
            scale = 0;
        }

        Ok(Self {
            schema,
            table,
            alias,
            name,
            r#type,
            flags: ColumnFlags::from_bits_truncate(flags),
            precision,
            scale,
        })
    }
}
