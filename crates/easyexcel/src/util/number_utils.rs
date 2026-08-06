//! Java `NumberUtils` 兼容路径。
//!
//! 可复用的 `DecimalFormat` 解析、舍入和数值转换算法位于
//! `easyexcel-format`；本模块仅保留 `EasyExcel` 错误类型适配。

use bigdecimal::BigDecimal;
use num_bigint::BigInt;

use crate::{ExcelError, NumberRoundingMode};

pub(crate) use easyexcel_format::NonFiniteNumber;
/// 对应 Java：NumberUtils.parseShort。
pub(crate) fn format_decimal(
    value: &BigDecimal,
    negative: bool,
    pattern: Option<&str>,
    rounding_mode: NumberRoundingMode,
) -> Result<String, ExcelError> {
    easyexcel_format::format_decimal(value, negative, pattern, rounding_mode).map_err(Into::into)
}
/// 对应 Java：NumberUtils.parseShort。
pub(crate) fn format_non_finite(
    value: NonFiniteNumber,
    pattern: Option<&str>,
) -> Result<String, ExcelError> {
    easyexcel_format::format_non_finite(value, pattern).map_err(Into::into)
}
/// 对应 Java：NumberUtils.parseShort。
pub(crate) fn parse_decimal(value: &str, pattern: Option<&str>) -> Result<BigDecimal, ExcelError> {
    easyexcel_format::parse_decimal(value, pattern).map_err(Into::into)
}

/// 对应 Java：`NumberUtils.parseShort`。
///
/// # Errors
///
/// 输入不是可表示的 `i16` 数值时返回格式错误。
pub fn parse_short(value: &str) -> Result<i16, ExcelError> {
    easyexcel_format::parse_short(value).map_err(Into::into)
}

/// 对应 Java：`NumberUtils.parseLong`。
///
/// # Errors
///
/// 输入不是可表示的 `i64` 数值时返回格式错误。
pub fn parse_long(value: &str) -> Result<i64, ExcelError> {
    easyexcel_format::parse_long(value).map_err(Into::into)
}

/// 对应 Java：`NumberUtils.parseInteger`。
///
/// # Errors
///
/// 输入不是可表示的 `i32` 数值时返回格式错误。
pub fn parse_integer(value: &str) -> Result<i32, ExcelError> {
    easyexcel_format::parse_integer(value).map_err(Into::into)
}

/// 对应 Java：`NumberUtils.parseFloat`。
///
/// # Errors
///
/// 输入不是有效的单精度浮点数时返回格式错误。
pub fn parse_float(value: &str) -> Result<f32, ExcelError> {
    easyexcel_format::parse_float(value).map_err(Into::into)
}

/// 对应 Java：`NumberUtils.parseBigDecimal`。
///
/// # Errors
///
/// 输入不是有效的任意精度十进制数时返回格式错误。
pub fn parse_big_decimal(value: &str) -> Result<BigDecimal, ExcelError> {
    easyexcel_format::parse_big_decimal(value).map_err(Into::into)
}

/// 对应 Java：`NumberUtils.parseByte`。
///
/// # Errors
///
/// 输入不是可表示的 `i8` 数值时返回格式错误。
pub fn parse_byte(value: &str) -> Result<i8, ExcelError> {
    easyexcel_format::parse_byte(value).map_err(Into::into)
}

/// 对应 Java：`NumberUtils.parseDouble`。
///
/// # Errors
///
/// 输入不是有效的双精度浮点数时返回格式错误。
pub fn parse_double(value: &str) -> Result<f64, ExcelError> {
    easyexcel_format::parse_double(value).map_err(Into::into)
}

/// 对应 Java：NumberUtils.parseShort。 对应 Apache Commons：`NumberUtils.createBigInteger`。
///
/// # Errors
///
/// 输入不是有效的任意精度整数时返回格式错误。
pub fn parse_big_int(value: &str) -> Result<BigInt, ExcelError> {
    easyexcel_format::parse_big_int(value).map_err(Into::into)
}
