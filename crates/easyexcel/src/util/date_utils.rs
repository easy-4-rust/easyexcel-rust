//! Java `DateUtils` 兼容入口。
//!
//! 日期模式解析与日期换算由 `easyexcel-model` 提供；本模块只保留
//! `EasyExcel` Java 风格的方法名称和错误适配。

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use bigdecimal::BigDecimal;
use std::sync::{LazyLock, RwLock};

use crate::core::excel_error::ExcelError;

pub use easyexcel_model::{
    DATE_FORMAT_10, DATE_FORMAT_14, DATE_FORMAT_16, DATE_FORMAT_16_FORWARD_SLASH, DATE_FORMAT_17,
    DATE_FORMAT_19, DATE_FORMAT_19_FORWARD_SLASH, DAY_MILLISECONDS, DEFAULT_DATE_FORMAT,
    DEFAULT_LOCAL_DATE_FORMAT, HOURS_PER_DAY, MINUTES_PER_HOUR, SECONDS_PER_DAY,
    SECONDS_PER_MINUTE,
};

/// Java `DateUtils.defaultDateFormat` 的线程安全运行时配置。
pub static DEFAULT_DATE_FORMAT_SETTING: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new(DEFAULT_DATE_FORMAT.to_owned()));

/// Java `DateUtils.defaultLocalDateFormat` 的线程安全运行时配置。
pub static DEFAULT_LOCAL_DATE_FORMAT_SETTING: LazyLock<RwLock<String>> =
    LazyLock::new(|| RwLock::new(DEFAULT_LOCAL_DATE_FORMAT.to_owned()));

/// 返回当前默认日期时间格式。
#[must_use]
pub fn default_date_format() -> String {
    DEFAULT_DATE_FORMAT_SETTING
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// 修改后续日期转换使用的默认日期时间格式。
pub fn set_default_date_format(value: impl Into<String>) {
    *DEFAULT_DATE_FORMAT_SETTING
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = value.into();
}

/// 返回当前默认本地日期格式。
#[must_use]
pub fn default_local_date_format() -> String {
    DEFAULT_LOCAL_DATE_FORMAT_SETTING
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// 修改后续本地日期转换使用的默认格式。
pub fn set_default_local_date_format(value: impl Into<String>) {
    *DEFAULT_LOCAL_DATE_FORMAT_SETTING
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = value.into();
}

/// Java `com.alibaba.excel.util.DateUtils` 的静态门面。
///
/// Rust 模块级函数继续保留给既有调用方；该零尺寸类型提供与 Java 静态工具类
/// 一致的 `DateUtils::method(...)` 调用入口。
#[derive(Debug, Clone, Copy, Default)]
pub struct DateUtils;

impl DateUtils {
    /// 对应 Java：`DateUtils.DATE_FORMAT_10`。
    pub const DATE_FORMAT_10: &'static str = DATE_FORMAT_10;
    /// 对应 Java：`DateUtils.DATE_FORMAT_14`。
    pub const DATE_FORMAT_14: &'static str = DATE_FORMAT_14;
    /// 对应 Java：`DateUtils.DATE_FORMAT_16`。
    pub const DATE_FORMAT_16: &'static str = DATE_FORMAT_16;
    /// 对应 Java：`DateUtils.DATE_FORMAT_16_FORWARD_SLASH`。
    pub const DATE_FORMAT_16_FORWARD_SLASH: &'static str = DATE_FORMAT_16_FORWARD_SLASH;
    /// 对应 Java：`DateUtils.DATE_FORMAT_17`。
    pub const DATE_FORMAT_17: &'static str = DATE_FORMAT_17;
    /// 对应 Java：`DateUtils.DATE_FORMAT_19`。
    pub const DATE_FORMAT_19: &'static str = DATE_FORMAT_19;
    /// 对应 Java：`DateUtils.DATE_FORMAT_19_FORWARD_SLASH`。
    pub const DATE_FORMAT_19_FORWARD_SLASH: &'static str = DATE_FORMAT_19_FORWARD_SLASH;
    /// 对应 Java：`DateUtils.defaultDateFormat`。
    pub const DEFAULT_DATE_FORMAT: &'static str = DEFAULT_DATE_FORMAT;
    /// 对应 Java：`DateUtils.defaultLocalDateFormat`。
    pub const DEFAULT_LOCAL_DATE_FORMAT: &'static str = DEFAULT_LOCAL_DATE_FORMAT;
    /// 对应 Java：`DateUtils.SECONDS_PER_MINUTE`。
    pub const SECONDS_PER_MINUTE: i32 = SECONDS_PER_MINUTE;
    /// 对应 Java：`DateUtils.MINUTES_PER_HOUR`。
    pub const MINUTES_PER_HOUR: i32 = MINUTES_PER_HOUR;
    /// 对应 Java：`DateUtils.HOURS_PER_DAY`。
    pub const HOURS_PER_DAY: i32 = HOURS_PER_DAY;
    /// 对应 Java：`DateUtils.SECONDS_PER_DAY`。
    pub const SECONDS_PER_DAY: i32 = SECONDS_PER_DAY;
    /// 对应 Java：`DateUtils.DAY_MILLISECONDS`。
    pub const DAY_MILLISECONDS: i64 = DAY_MILLISECONDS;

    /// 返回当前 Java `defaultDateFormat` 配置。
    #[must_use]
    pub fn default_date_format() -> String { default_date_format() }

    /// 修改 Java `defaultDateFormat` 的 Rust 运行时配置。
    pub fn set_default_date_format(value: impl Into<String>) { set_default_date_format(value); }

    /// 返回当前 Java `defaultLocalDateFormat` 配置。
    #[must_use]
    pub fn default_local_date_format() -> String { default_local_date_format() }

    /// 修改 Java `defaultLocalDateFormat` 的 Rust 运行时配置。
    pub fn set_default_local_date_format(value: impl Into<String>) {
        set_default_local_date_format(value);
    }

    /// 解析 Java 日期字符串；空格式时沿用 Java 的长度推断规则。
    pub fn parse_date(
        date_string: &str,
        date_format: Option<&str>,
    ) -> Result<NaiveDateTime, ExcelError> {
        let date_format = date_format.map_or_else(|| switch_date_format(date_string), Ok)?;
        parse_date(date_string, [date_format])
    }
    /// Java `parseDate(String)` 重载。
    pub fn parse_date_default(date_string: &str) -> Result<NaiveDateTime, ExcelError> {
        Self::parse_date(date_string, None)
    }

    /// 解析本地日期时间；Rust 的 `chrono` 格式化不依赖 JVM Locale。
    pub fn parse_local_date_time(
        date_string: &str,
        date_format: Option<&str>,
    ) -> Result<NaiveDateTime, ExcelError> {
        parse_local_date_time(date_string, date_format)
    }

    /// 解析本地日期；Rust 的 `chrono` 格式化不依赖 JVM Locale。
    pub fn parse_local_date(
        date_string: &str,
        date_format: Option<&str>,
    ) -> Result<NaiveDate, ExcelError> {
        parse_local_date(date_string, date_format)
    }

    /// 按 Java 日期模式格式化本地日期时间。
    #[must_use]
    pub fn format(date: NaiveDateTime, date_format: Option<&str>) -> String {
        let default_format;
        let date_format = match date_format {
            Some(value) => value,
            None => {
                default_format = default_date_format();
                &default_format
            }
        };
        format(date, date_format)
    }

    /// 按 Java 日期模式格式化本地日期。
    #[must_use]
    pub fn format_local_date(date: NaiveDate, date_format: Option<&str>) -> String {
        let default_format;
        let date_format = match date_format {
            Some(value) => value,
            None => {
                default_format = default_local_date_format();
                &default_format
            }
        };
        date.format(&easyexcel_model::chrono_date_format(date_format)).to_string()
    }
    /// Java `format(Date)` 重载。
    #[must_use]
    pub fn format_default(date: NaiveDateTime) -> String { Self::format(date, None) }
    /// Java Locale 重载；chrono 模式本身保持确定性，locale 作为显式兼容参数保留。
    #[must_use]
    pub fn format_with_locale(date: NaiveDateTime, date_format: Option<&str>, _locale: &str) -> String {
        Self::format(date, date_format)
    }
    /// Java LocalDate Locale 重载。
    #[must_use]
    pub fn format_local_date_with_locale(date: NaiveDate, date_format: Option<&str>, _locale: &str) -> String {
        Self::format_local_date(date, date_format)
    }
    /// 按 Excel serial 和日期窗口格式化 BigDecimal。
    #[must_use]
    pub fn format_decimal(value: &BigDecimal, use_1904_windowing: bool, date_format: Option<&str>) -> String {
        value.to_string().parse::<f64>().ok()
            .and_then(|serial| Self::get_local_date_time(serial, use_1904_windowing))
            .map_or_else(String::new, |date| Self::format(date, date_format))
    }

    /// 根据字符串形态选择 Java 日期格式。
    pub fn switch_date_format(date_string: &str) -> Result<&'static str, ExcelError> {
        switch_date_format(date_string)
    }

    /// 将 Excel serial 转为 Java `Date` 对应的本地日期时间。
    #[must_use]
    pub fn get_java_date(date: f64, use_1904_windowing: bool) -> Option<NaiveDateTime> {
        Self::get_java_calendar(date, use_1904_windowing, None, true)
    }
    /// Java `getJavaCalendar` 的后端中立日历值。
    ///
    /// `time_zone` 是 Java `Calendar` 的展示属性；Rust 返回无时区的
    /// `NaiveDateTime`，因此保留参数形状但不伪造时区转换。秒舍入语义完整保留。
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn get_java_calendar(
        date: f64,
        use_1904_windowing: bool,
        _time_zone: Option<&str>,
        round_seconds: bool,
    ) -> Option<NaiveDateTime> {
        if !Self::is_valid_excel_date(date) {
            return None;
        }
        let whole_days = date.floor();
        let whole_days = i32::try_from(whole_days as i64).ok()?;
        let milliseconds_in_day = ((date - f64::from(whole_days))
            * DAY_MILLISECONDS as f64
            + 0.5)
            .floor() as i32;
        Self::set_calendar(
            whole_days,
            milliseconds_in_day,
            use_1904_windowing,
            round_seconds,
        )
    }
    /// Java `setCalendar` 的纯函数等价：从整日与毫秒偏移构造值。
    #[must_use]
    pub fn set_calendar(
        whole_days: i32,
        milliseconds_in_day: i32,
        use_1904_windowing: bool,
        round_seconds: bool,
    ) -> Option<NaiveDateTime> {
        easyexcel_model::excel_parts_to_datetime(
            whole_days,
            milliseconds_in_day,
            use_1904_windowing,
            round_seconds,
        )
    }

    /// 将 Excel serial 转为本地日期时间。
    #[must_use]
    pub fn get_local_date_time(date: f64, use_1904_windowing: bool) -> Option<NaiveDateTime> {
        get_local_date_time(date, use_1904_windowing)
    }

    /// 将 Excel serial 转为本地日期。
    #[must_use]
    pub fn get_local_date(date: f64, use_1904_windowing: bool) -> Option<NaiveDate> {
        get_local_date(date, use_1904_windowing)
    }

    /// 判断 Excel serial 是否有效。
    #[must_use]
    pub fn is_valid_excel_date(value: f64) -> bool {
        is_valid_excel_date(value)
    }

    /// 判断指定格式编号/格式串是否为日期格式。
    #[must_use]
    pub fn is_a_date_format(format_index: i32, format_string: Option<&str>) -> bool {
        is_a_date_format(format_index, format_string)
    }

    /// Java 名称 `isADateFormat` 的机械 snake_case 兼容入口。
    #[must_use]
    pub fn is_adate_format(format_index: i32, format_string: Option<&str>) -> bool {
        Self::is_a_date_format(format_index, format_string)
    }

    /// 绕过 Java ThreadLocal 缓存执行日期格式判断。
    #[must_use]
    pub fn is_a_date_format_uncached(format_index: i32, format_string: Option<&str>) -> bool {
        is_a_date_format_uncached(format_index, format_string)
    }

    /// Java 名称 `isADateFormatUncached` 的机械 snake_case 兼容入口。
    #[must_use]
    pub fn is_adate_format_uncached(format_index: i32, format_string: Option<&str>) -> bool {
        Self::is_a_date_format_uncached(format_index, format_string)
    }

    /// 判断内建格式编号是否为 Excel 内部日期格式。
    #[must_use]
    pub fn is_internal_date_format(format_index: i32) -> bool {
        u16::try_from(format_index)
            .ok()
            .is_some_and(|index| easyexcel_model::numfmt::is_date_format_id(index, None))
    }

    /// 清理 Java ThreadLocal 缓存的兼容生命周期钩子。
    pub const fn remove_thread_local_cache() {
        remove_thread_local_cache();
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按多个 Java 日期模式依次解析。
///
/// # Errors
///
/// 所有候选模式均无法解析输入时返回格式错误。
pub fn parse_date<'a>(
    value: &str,
    parse_patterns: impl IntoIterator<Item = &'a str>,
) -> Result<NaiveDateTime, ExcelError> {
    easyexcel_model::dates::parse_java_date(value, parse_patterns)
        .map_err(|error| ExcelError::Format(error.to_string()))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 使用 Java 日期模式格式化日期时间。
#[must_use]
pub fn format(date: NaiveDateTime, pattern: &str) -> String {
    easyexcel_model::dates::format_java_date(date, pattern)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将 Excel 整数天数转换为 UTC 时间。
#[must_use]
pub fn get_java_date(days: i64) -> DateTime<Utc> {
    easyexcel_model::dates::excel_days_to_utc(days)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按内建格式索引和可选自定义格式判断日期格式。
#[must_use]
pub fn is_a_date_format(format_index: i32, format_string: Option<&str>) -> bool {
    u16::try_from(format_index)
        .ok()
        .is_some_and(|index| easyexcel_model::numfmt::is_date_format_id(index, format_string))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断自定义格式代码是否表达日期或时间。
#[must_use]
pub fn is_internal_date_format(format: &str) -> bool {
    easyexcel_model::dates::is_internal_date_format(format)
}

/// 根据 Java 的长度/分隔符规则推断日期格式。
pub fn switch_date_format(value: &str) -> Result<&'static str, ExcelError> {
    easyexcel_model::infer_java_date_pattern(value)
        .map_err(|error| ExcelError::Format(error.to_string()))
}

/// 按显式或推断格式解析本地日期时间。
pub fn parse_local_date_time(
    value: &str,
    pattern: Option<&str>,
) -> Result<NaiveDateTime, ExcelError> {
    let pattern = pattern.map_or_else(|| switch_date_format(value), Ok)?;
    parse_date(value, [pattern])
}

/// 按显式或推断格式解析本地日期。
pub fn parse_local_date(value: &str, pattern: Option<&str>) -> Result<NaiveDate, ExcelError> {
    let pattern = pattern.map_or_else(|| switch_date_format(value), Ok)?;
    let chrono_pattern = easyexcel_model::chrono_date_format(pattern);
    NaiveDate::parse_from_str(value, &chrono_pattern)
        .map_err(|error| ExcelError::Format(error.to_string()))
}

/// 将 Excel serial 转换为本地日期时间。
#[must_use]
pub fn get_local_date_time(value: f64, use_1904_windowing: bool) -> Option<NaiveDateTime> {
    let system = if use_1904_windowing {
        easyexcel_model::DateSystem::Date1904
    } else {
        easyexcel_model::DateSystem::Date1900
    };
    system.serial_to_datetime(value)
}

/// 将 Excel serial 转换为本地日期。
#[must_use]
pub fn get_local_date(value: f64, use_1904_windowing: bool) -> Option<NaiveDate> {
    get_local_date_time(value, use_1904_windowing).map(|value| value.date())
}

/// 判断 Excel serial 是否有效。
#[must_use]
pub fn is_valid_excel_date(value: f64) -> bool {
    value >= 0.0
}

/// 未命中线程本地缓存的日期格式判断；Rust 无缓存，因此复用权威实现。
#[must_use]
pub fn is_a_date_format_uncached(format_index: i32, format_string: Option<&str>) -> bool {
    is_a_date_format(format_index, format_string)
}

/// Java 清理 `ThreadLocal<Format>` 的兼容生命周期钩子。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn remove_thread_local_cache() {
    // Rust 日期格式化器不持有 JVM ThreadLocal 状态，因此无需释放资源。
}
