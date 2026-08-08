//! Java `NumberUtils` 兼容路径。
//!
//! 可复用的 `DecimalFormat` 解析、舍入和数值转换算法位于
//! `easyexcel-format`；本模块仅保留 `EasyExcel` 错误类型适配。

use bigdecimal::BigDecimal;
use num_bigint::BigInt;

use crate::{
    CellValue, ExcelContentProperty, ExcelError, NumberRoundingMode, WriteCellData,
};

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

fn number_format(property: Option<&ExcelContentProperty>) -> Option<(&str, NumberRoundingMode)> {
    property.and_then(|property| {
        property
            .number_format_property
            .as_ref()
            .map(|format| (format.format(), format.rounding_mode()))
            .or_else(|| {
                property
                    .number_format
                    .map(|format| (format, NumberRoundingMode::default()))
            })
    })
}

/// 对应 Java：`NumberUtils.format(Number, ExcelContentProperty)`。
///
/// # Errors
///
/// 数值格式非法或指定了不允许舍入的格式时返回格式错误。
pub fn format(
    value: &BigDecimal,
    property: Option<&ExcelContentProperty>,
) -> Result<String, ExcelError> {
    let Some((pattern, rounding_mode)) = number_format(property) else {
        return Ok(value.to_string());
    };
    format_decimal(value, value.to_string().starts_with('-'), Some(pattern), rounding_mode)
}

/// 对应 Java：`NumberUtils.formatToCellData(Number, ExcelContentProperty)`。
///
/// 保留数值单元格类型，并把注解数字格式附着到 `WriteCellData`。
#[must_use]
pub fn format_to_cell_data(
    value: &BigDecimal,
    property: Option<&ExcelContentProperty>,
) -> WriteCellData {
    let mut cell = WriteCellData::new(CellValue::Decimal(value.clone()));
    if let Some((pattern, _)) = number_format(property) {
        crate::util::work_book_util::fill_data_format(&mut cell, Some(pattern), "");
    }
    cell
}

/// 对应 Java：`NumberUtils.formatToCellDataString(Number, ExcelContentProperty)`。
///
/// # Errors
///
/// 数值格式非法时返回格式错误。
pub fn format_to_cell_data_string(
    value: &BigDecimal,
    property: Option<&ExcelContentProperty>,
) -> Result<WriteCellData, ExcelError> {
    Ok(WriteCellData::from_string(format(value, property)?))
}

/// 对应 Java：带 `ExcelContentProperty` 的 `parseBigDecimal` 重载。
pub fn parse_big_decimal_with_property(
    value: &str,
    property: Option<&ExcelContentProperty>,
) -> Result<BigDecimal, ExcelError> {
    parse_decimal(value, number_format(property).map(|(pattern, _)| pattern))
}

macro_rules! parse_with_property {
    ($name:ident, $target:ty, $convert:ident) => {
        #[doc = "对应 Java：带 `ExcelContentProperty` 的数字解析重载。"]
        pub fn $name(
            value: &str,
            property: Option<&ExcelContentProperty>,
        ) -> Result<$target, ExcelError> {
            let decimal = parse_big_decimal_with_property(value, property)?;
            easyexcel_format::$convert(&decimal.to_string()).map_err(Into::into)
        }
    };
}

parse_with_property!(parse_short_with_property, i16, parse_short);
parse_with_property!(parse_long_with_property, i64, parse_long);
parse_with_property!(parse_integer_with_property, i32, parse_integer);
parse_with_property!(parse_float_with_property, f32, parse_float);
parse_with_property!(parse_byte_with_property, i8, parse_byte);
parse_with_property!(parse_double_with_property, f64, parse_double);
