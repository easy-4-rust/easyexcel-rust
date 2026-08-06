//! Java `DateUtils` 兼容入口。
//!
//! 日期模式解析与日期换算由 `easyexcel-model` 提供；本模块只保留
//! `EasyExcel` Java 风格的方法名称和错误适配。

use chrono::{DateTime, NaiveDateTime, Utc};

use crate::core::excel_error::ExcelError;

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

/// Java 清理 `ThreadLocal<Format>` 的兼容生命周期钩子。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn remove_thread_local_cache() {
    // Rust 日期格式化器不持有 JVM ThreadLocal 状态，因此无需释放资源。
}
