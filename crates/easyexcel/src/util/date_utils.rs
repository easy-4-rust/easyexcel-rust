//! Java `DateUtils` 兼容入口。
//!
//! 日期模式解析与日期换算由 `easyexcel-model` 提供；本模块只保留
//! `EasyExcel` Java 风格的方法名称和错误适配。

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
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
    pub fn default_date_format() -> String {
        default_date_format()
    }

    /// 修改 Java `defaultDateFormat` 的 Rust 运行时配置。
    pub fn set_default_date_format(value: impl Into<String>) {
        set_default_date_format(value);
    }

    /// 返回当前 Java `defaultLocalDateFormat` 配置。
    #[must_use]
    pub fn default_local_date_format() -> String {
        default_local_date_format()
    }

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
        date.format(&easyexcel_model::chrono_date_format(date_format))
            .to_string()
    }
    /// Java `format(Date)` 重载。
    #[must_use]
    pub fn format_default(date: NaiveDateTime) -> String {
        Self::format(date, None)
    }
    /// Java Locale 重载；chrono 模式本身保持确定性，locale 作为显式兼容参数保留。
    #[must_use]
    pub fn format_with_locale(
        date: NaiveDateTime,
        date_format: Option<&str>,
        _locale: &str,
    ) -> String {
        Self::format(date, date_format)
    }
    /// Java LocalDate Locale 重载。
    #[must_use]
    pub fn format_local_date_with_locale(
        date: NaiveDate,
        date_format: Option<&str>,
        _locale: &str,
    ) -> String {
        Self::format_local_date(date, date_format)
    }
    /// 按 Excel serial 和日期窗口格式化 BigDecimal。
    #[must_use]
    pub fn format_decimal(
        value: &BigDecimal,
        use_1904_windowing: bool,
        date_format: Option<&str>,
    ) -> String {
        value
            .to_string()
            .parse::<f64>()
            .ok()
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
        let milliseconds_in_day =
            ((date - f64::from(whole_days)) * DAY_MILLISECONDS as f64 + 0.5).floor() as i32;
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

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    // ---- 常量测试 ----

    #[test]
    fn date_utils_struct_constants_match_module_constants() {
        assert_eq!(DateUtils::DATE_FORMAT_10, DATE_FORMAT_10);
        assert_eq!(DateUtils::DATE_FORMAT_14, DATE_FORMAT_14);
        assert_eq!(DateUtils::DATE_FORMAT_16, DATE_FORMAT_16);
        assert_eq!(
            DateUtils::DATE_FORMAT_16_FORWARD_SLASH,
            DATE_FORMAT_16_FORWARD_SLASH
        );
        assert_eq!(DateUtils::DATE_FORMAT_17, DATE_FORMAT_17);
        assert_eq!(DateUtils::DATE_FORMAT_19, DATE_FORMAT_19);
        assert_eq!(
            DateUtils::DATE_FORMAT_19_FORWARD_SLASH,
            DATE_FORMAT_19_FORWARD_SLASH
        );
        assert_eq!(DateUtils::DEFAULT_DATE_FORMAT, DEFAULT_DATE_FORMAT);
        assert_eq!(
            DateUtils::DEFAULT_LOCAL_DATE_FORMAT,
            DEFAULT_LOCAL_DATE_FORMAT
        );
        assert_eq!(DateUtils::SECONDS_PER_MINUTE, SECONDS_PER_MINUTE);
        assert_eq!(DateUtils::MINUTES_PER_HOUR, MINUTES_PER_HOUR);
        assert_eq!(DateUtils::HOURS_PER_DAY, HOURS_PER_DAY);
        assert_eq!(DateUtils::SECONDS_PER_DAY, SECONDS_PER_DAY);
        assert_eq!(DateUtils::DAY_MILLISECONDS, DAY_MILLISECONDS);
    }

    #[test]
    fn date_format_10_is_yyyy_mm_dd() {
        assert_eq!(DATE_FORMAT_10, "yyyy-MM-dd");
    }

    #[test]
    fn date_format_19_is_full_datetime() {
        assert_eq!(DATE_FORMAT_19, "yyyy-MM-dd HH:mm:ss");
    }

    #[test]
    fn seconds_per_day_is_86400() {
        assert_eq!(SECONDS_PER_DAY, 86_400);
    }

    #[test]
    fn day_milliseconds_is_86400000() {
        assert_eq!(DAY_MILLISECONDS, 86_400_000);
    }

    // ---- 默认日期格式设置 ----

    #[test]
    fn default_date_format_returns_initial_value() {
        let fmt = default_date_format();
        assert_eq!(fmt, DEFAULT_DATE_FORMAT);
    }

    #[test]
    fn set_default_date_format_changes_value() {
        let original = default_date_format();
        set_default_date_format("yyyy/MM/dd");
        assert_eq!(default_date_format(), "yyyy/MM/dd");
        set_default_date_format(original);
    }

    #[test]
    fn default_local_date_format_returns_initial_value() {
        let fmt = default_local_date_format();
        assert_eq!(fmt, DEFAULT_LOCAL_DATE_FORMAT);
    }

    #[test]
    fn set_default_local_date_format_changes_value() {
        let original = default_local_date_format();
        set_default_local_date_format("MM/dd/yyyy");
        assert_eq!(default_local_date_format(), "MM/dd/yyyy");
        set_default_local_date_format(original);
    }

    // ---- DateUtils 结构体方法 ----

    #[test]
    fn date_utils_default_date_format() {
        let fmt = DateUtils::default_date_format();
        assert_eq!(fmt, DEFAULT_DATE_FORMAT);
    }

    #[test]
    fn date_utils_set_default_date_format() {
        let original = DateUtils::default_date_format();
        DateUtils::set_default_date_format("yyyyMMdd");
        assert_eq!(DateUtils::default_date_format(), "yyyyMMdd");
        DateUtils::set_default_date_format(original);
    }

    #[test]
    fn date_utils_default_local_date_format() {
        let fmt = DateUtils::default_local_date_format();
        assert_eq!(fmt, DEFAULT_LOCAL_DATE_FORMAT);
    }

    #[test]
    fn date_utils_set_default_local_date_format() {
        let original = DateUtils::default_local_date_format();
        DateUtils::set_default_local_date_format("dd/MM/yyyy");
        assert_eq!(DateUtils::default_local_date_format(), "dd/MM/yyyy");
        DateUtils::set_default_local_date_format(original);
    }

    #[test]
    fn date_utils_default_derives_correctly() {
        let _ = DateUtils::default();
    }

    #[test]
    fn date_utils_clone_copy() {
        let a = DateUtils;
        let _b = a;
        let _c = a.clone();
    }

    // ---- is_valid_excel_date ----

    #[test]
    fn is_valid_excel_date_positive() {
        assert!(is_valid_excel_date(1.0));
        assert!(is_valid_excel_date(44000.0));
    }

    #[test]
    fn is_valid_excel_date_zero() {
        assert!(is_valid_excel_date(0.0));
    }

    #[test]
    fn is_valid_excel_date_negative() {
        assert!(!is_valid_excel_date(-1.0));
    }

    #[test]
    fn date_utils_is_valid_excel_date_delegates() {
        assert!(DateUtils::is_valid_excel_date(44000.0));
        assert!(!DateUtils::is_valid_excel_date(-1.0));
    }

    // ---- switch_date_format ----

    #[test]
    fn switch_date_format_10_chars() {
        // "2024-01-01" 10 个字符 → DATE_FORMAT_10
        let result = switch_date_format("2024-01-01").unwrap();
        assert_eq!(result, DATE_FORMAT_10);
    }

    #[test]
    fn switch_date_format_19_chars() {
        // "2024-01-01 12:00:00" 19 个字符 → DATE_FORMAT_19
        let result = switch_date_format("2024-01-01 12:00:00").unwrap();
        assert_eq!(result, DATE_FORMAT_19);
    }

    // ---- parse_date / format ----

    #[test]
    fn parse_date_with_explicit_format() {
        let result = parse_date("2024-01-15", [DATE_FORMAT_10]);
        assert!(result.is_ok(), "解析日期失败: {:?}", result.err());
        let dt = result.unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-01-15");
    }

    #[test]
    fn parse_date_with_format_19() {
        let result = parse_date("2024-01-15 10:30:00", [DATE_FORMAT_19]);
        assert!(result.is_ok(), "解析日期失败: {:?}", result.err());
    }

    #[test]
    fn parse_date_invalid_format_returns_error() {
        let result = parse_date("not-a-date", [DATE_FORMAT_10]);
        assert!(result.is_err());
    }

    #[test]
    fn format_date_roundtrip() {
        let dt = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();
        let formatted = format(dt, DATE_FORMAT_19);
        assert_eq!(formatted, "2024-06-15 10:30:00");
    }

    #[test]
    fn format_date_format_10() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let formatted = format(dt, DATE_FORMAT_10);
        assert_eq!(formatted, "2024-01-01");
    }

    // ---- DateUtils::parse_date / parse_date_default ----

    #[test]
    fn date_utils_parse_date_with_format() {
        let result = DateUtils::parse_date("2024-03-20", Some(DATE_FORMAT_10));
        assert!(result.is_ok(), "解析失败: {:?}", result.err());
    }

    #[test]
    fn date_utils_parse_date_default_infers_format() {
        // "2024-01-01" 10 字符应推断为 DATE_FORMAT_10
        let result = DateUtils::parse_date_default("2024-01-01");
        assert!(result.is_ok(), "推断格式解析失败: {:?}", result.err());
    }

    #[test]
    fn date_utils_format_with_explicit_format() {
        let dt = NaiveDate::from_ymd_opt(2024, 12, 25)
            .unwrap()
            .and_hms_opt(8, 0, 0)
            .unwrap();
        let result = DateUtils::format(dt, Some(DATE_FORMAT_19));
        assert_eq!(result, "2024-12-25 08:00:00");
    }

    #[test]
    fn date_utils_format_default() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let result = DateUtils::format_default(dt);
        assert!(!result.is_empty());
    }

    #[test]
    fn date_utils_format_with_locale() {
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        let result = DateUtils::format_with_locale(dt, Some(DATE_FORMAT_10), "en_US");
        assert_eq!(result, "2024-01-01");
    }

    // ---- DateUtils::parse_local_date_time / parse_local_date ----

    #[test]
    fn date_utils_parse_local_date_time() {
        let result = DateUtils::parse_local_date_time("2024-01-01 12:00:00", Some(DATE_FORMAT_19));
        assert!(result.is_ok(), "解析失败: {:?}", result.err());
    }

    #[test]
    fn date_utils_parse_local_date() {
        let result = DateUtils::parse_local_date("2024-01-01", Some(DATE_FORMAT_10));
        assert!(result.is_ok(), "解析失败: {:?}", result.err());
    }

    // ---- format_local_date ----

    #[test]
    fn date_utils_format_local_date() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = DateUtils::format_local_date(date, Some(DATE_FORMAT_10));
        assert_eq!(result, "2024-06-15");
    }

    #[test]
    fn date_utils_format_local_date_with_locale() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 15).unwrap();
        let result = DateUtils::format_local_date_with_locale(date, Some(DATE_FORMAT_10), "zh_CN");
        assert_eq!(result, "2024-06-15");
    }

    // ---- get_java_date (serial) ----

    #[test]
    fn get_java_date_from_serial() {
        // Excel serial 44927 = 2023-01-01
        let result = get_java_date(44927);
        assert_eq!(result.format("%Y-%m-%d").to_string(), "2023-01-01");
    }

    // ---- get_local_date_time / get_local_date ----

    #[test]
    fn get_local_date_time_1900_system() {
        // Excel serial 2 = 1900-01-01（1900 日期系统）
        let result = get_local_date_time(2.0, false);
        assert!(result.is_some(), "serial 2 应为有效日期");
    }

    #[test]
    fn get_local_date_time_1904_system() {
        let result = get_local_date_time(2.0, true);
        assert!(result.is_some());
    }

    #[test]
    fn get_local_date_time_negative_is_none() {
        let result = get_local_date_time(-1.0, false);
        assert!(result.is_none());
    }

    #[test]
    fn get_local_date_from_serial() {
        let result = get_local_date(44927.0, false);
        assert!(result.is_some(), "serial 44927 应为有效日期");
    }

    // ---- DateUtils::get_java_date / get_java_calendar ----

    #[test]
    fn date_utils_get_java_date_valid() {
        let result = DateUtils::get_java_date(44927.0, false);
        assert!(result.is_some());
    }

    #[test]
    fn date_utils_get_java_date_invalid() {
        let result = DateUtils::get_java_date(-1.0, false);
        assert!(result.is_none());
    }

    #[test]
    fn date_utils_get_java_calendar_with_rounding() {
        let result = DateUtils::get_java_calendar(44927.5, false, None, true);
        assert!(result.is_some());
    }

    #[test]
    fn date_utils_get_java_calendar_without_rounding() {
        let result = DateUtils::get_java_calendar(44927.5, false, None, false);
        assert!(result.is_some());
    }

    // ---- DateUtils::set_calendar ----

    #[test]
    fn set_calendar_basic() {
        let result = DateUtils::set_calendar(2, 0, false, true);
        assert!(result.is_some());
    }

    // ---- is_a_date_format ----

    #[test]
    fn is_a_date_format_builtin_date_formats() {
        // 内建格式 14 = yyyy-MM-dd HH:mm:ss，应为日期格式
        assert!(is_a_date_format(14, None));
    }

    #[test]
    fn is_a_date_format_general_format_is_not_date() {
        // 内建格式 0 = General，不是日期格式
        assert!(!is_a_date_format(0, None));
    }

    #[test]
    fn date_utils_is_a_date_format_delegates() {
        assert!(DateUtils::is_a_date_format(14, None));
        assert!(!DateUtils::is_a_date_format(0, None));
    }

    #[test]
    fn date_utils_is_adate_format_alias() {
        assert_eq!(
            DateUtils::is_adate_format(14, None),
            DateUtils::is_a_date_format(14, None)
        );
    }

    // ---- is_internal_date_format ----

    #[test]
    fn is_internal_date_format_date_pattern() {
        assert!(is_internal_date_format("yyyy-MM-dd"));
    }

    #[test]
    fn is_internal_date_format_non_date_pattern() {
        assert!(!is_internal_date_format("0.00"));
    }

    #[test]
    fn date_utils_is_internal_date_format_delegates() {
        assert!(DateUtils::is_internal_date_format(14));
    }

    // ---- is_a_date_format_uncached ----

    #[test]
    fn is_a_date_format_uncached_matches_cached() {
        assert_eq!(
            is_a_date_format_uncached(14, None),
            is_a_date_format(14, None)
        );
    }

    #[test]
    fn date_utils_is_adate_format_uncached_alias() {
        assert_eq!(
            DateUtils::is_adate_format_uncached(14, None),
            DateUtils::is_a_date_format_uncached(14, None)
        );
    }

    // ---- format_decimal ----

    #[test]
    fn format_decimal_valid_serial() {
        let dec = BigDecimal::from(44927_i32);
        let result = DateUtils::format_decimal(&dec, false, Some(DATE_FORMAT_10));
        assert!(!result.is_empty(), "格式化 BigDecimal 不应为空");
    }

    #[test]
    fn format_decimal_invalid_serial_returns_empty() {
        let dec = BigDecimal::from(-1_i32);
        let result = DateUtils::format_decimal(&dec, false, Some(DATE_FORMAT_10));
        assert!(result.is_empty(), "无效 serial 应返回空字符串");
    }

    // ---- parse_local_date_time / parse_local_date (模块级) ----

    #[test]
    fn parse_local_date_time_module_level() {
        let result = parse_local_date_time("2024-01-01 12:00:00", Some(DATE_FORMAT_19));
        assert!(result.is_ok());
    }

    #[test]
    fn parse_local_date_time_inferred_format() {
        let result = parse_local_date_time("2024-01-01", None);
        assert!(result.is_ok());
    }

    #[test]
    fn parse_local_date_module_level() {
        let result = parse_local_date("2024-01-01", Some(DATE_FORMAT_10));
        assert!(result.is_ok());
    }

    #[test]
    fn parse_local_date_inferred_format() {
        let result = parse_local_date("2024-01-01", None);
        assert!(result.is_ok());
    }

    // ---- remove_thread_local_cache ----

    #[test]
    fn remove_thread_local_cache_does_not_panic() {
        remove_thread_local_cache();
    }

    #[test]
    fn date_utils_remove_thread_local_cache_does_not_panic() {
        DateUtils::remove_thread_local_cache();
    }

    // ---- switch_date_format (via DateUtils) ----

    #[test]
    fn date_utils_switch_date_format_10_chars() {
        let result = DateUtils::switch_date_format("2024-01-01").unwrap();
        assert_eq!(result, DATE_FORMAT_10);
    }

    #[test]
    fn date_utils_switch_date_format_19_chars() {
        let result = DateUtils::switch_date_format("2024-01-01 12:00:00").unwrap();
        assert_eq!(result, DATE_FORMAT_19);
    }
}
