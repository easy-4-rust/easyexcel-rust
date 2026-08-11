//! Java `NumberUtils` 兼容路径。
//!
//! 可复用的 `DecimalFormat` 解析、舍入和数值转换算法位于
//! `easyexcel-format`；本模块仅保留 `EasyExcel` 错误类型适配。

use bigdecimal::BigDecimal;
use num_bigint::BigInt;

use crate::{CellValue, ExcelContentProperty, ExcelError, NumberRoundingMode, WriteCellData};

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
    format_decimal(
        value,
        value.to_string().starts_with('-'),
        Some(pattern),
        rounding_mode,
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // --- parse_short ---
    #[test]
    fn parse_short_valid_values() {
        assert_eq!(parse_short("0").unwrap(), 0);
        assert_eq!(parse_short("42").unwrap(), 42);
        assert_eq!(parse_short("-100").unwrap(), -100);
        assert_eq!(parse_short("32767").unwrap(), i16::MAX);
        assert_eq!(parse_short("-32768").unwrap(), i16::MIN);
    }

    #[test]
    fn parse_short_invalid_values() {
        assert!(parse_short("abc").is_err());
    }

    // --- parse_long ---
    #[test]
    fn parse_long_valid_values() {
        assert_eq!(parse_long("0").unwrap(), 0_i64);
        assert_eq!(parse_long("123456789").unwrap(), 123_456_789_i64);
        assert_eq!(parse_long("-999").unwrap(), -999_i64);
    }

    #[test]
    fn parse_long_invalid_values() {
        assert!(parse_long("abc").is_err());
        assert!(parse_long("").is_err());
    }

    // --- parse_integer ---
    #[test]
    fn parse_integer_valid_values() {
        assert_eq!(parse_integer("0").unwrap(), 0_i32);
        assert_eq!(parse_integer("42").unwrap(), 42_i32);
        assert_eq!(parse_integer("-1").unwrap(), -1_i32);
        assert_eq!(parse_integer("2147483647").unwrap(), i32::MAX);
    }

    #[test]
    fn parse_integer_invalid_values() {
        assert!(parse_integer("abc").is_err());
    }

    // --- parse_float ---
    #[test]
    fn parse_float_valid_values() {
        assert!((parse_float("3.14").unwrap() - 3.14_f32).abs() < f32::EPSILON);
        assert!((parse_float("-1.5").unwrap() - (-1.5_f32)).abs() < f32::EPSILON);
        assert!((parse_float("0").unwrap()).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_float_invalid_values() {
        assert!(parse_float("abc").is_err());
        assert!(parse_float("").is_err());
    }

    // --- parse_double ---
    #[test]
    fn parse_double_valid_values() {
        assert!((parse_double("3.14").unwrap() - 3.14_f64).abs() < f64::EPSILON);
        assert!((parse_double("-1.5").unwrap() - (-1.5_f64)).abs() < f64::EPSILON);
        assert!((parse_double("0").unwrap()).abs() < f64::EPSILON);
    }

    #[test]
    fn parse_double_invalid_values() {
        assert!(parse_double("abc").is_err());
    }

    // --- parse_byte ---
    #[test]
    fn parse_byte_valid_values() {
        assert_eq!(parse_byte("0").unwrap(), 0_i8);
        assert_eq!(parse_byte("127").unwrap(), i8::MAX);
        assert_eq!(parse_byte("-128").unwrap(), i8::MIN);
    }

    #[test]
    fn parse_byte_invalid_values() {
        assert!(parse_byte("abc").is_err());
    }

    // --- parse_big_decimal ---
    #[test]
    fn parse_big_decimal_valid_values() {
        let val = parse_big_decimal("123.456").unwrap();
        assert_eq!(val, BigDecimal::from_str("123.456").unwrap());
    }

    #[test]
    fn parse_big_decimal_invalid_values() {
        assert!(parse_big_decimal("abc").is_err());
    }

    // --- parse_big_int ---
    #[test]
    fn parse_big_int_valid_values() {
        let val = parse_big_int("99999999999999999999").unwrap();
        assert_eq!(val, BigInt::from_str("99999999999999999999").unwrap());
    }

    #[test]
    fn parse_big_int_invalid_values() {
        assert!(parse_big_int("abc").is_err());
    }

    // --- format ---
    #[test]
    fn format_returns_string_when_no_property() {
        let value = BigDecimal::from_str("123.45").unwrap();
        let result = format(&value, None).unwrap();
        assert_eq!(result, "123.45");
    }

    // --- format_to_cell_data ---
    #[test]
    fn format_to_cell_data_creates_decimal_cell() {
        let value = BigDecimal::from_str("42").unwrap();
        let cell = format_to_cell_data(&value, None);
        assert!(matches!(cell.value(), CellValue::Decimal(_)));
    }

    // --- format_to_cell_data_string ---
    #[test]
    fn format_to_cell_data_string_creates_string_cell() {
        let value = BigDecimal::from_str("42").unwrap();
        let cell = format_to_cell_data_string(&value, None).unwrap();
        assert!(matches!(cell.value(), CellValue::String(_)));
    }

    // --- parse_with_property (without property) ---
    #[test]
    fn parse_short_with_property_no_property() {
        assert_eq!(parse_short_with_property("42", None).unwrap(), 42_i16);
    }

    #[test]
    fn parse_long_with_property_no_property() {
        assert_eq!(parse_long_with_property("123", None).unwrap(), 123_i64);
    }

    #[test]
    fn parse_integer_with_property_no_property() {
        assert_eq!(parse_integer_with_property("42", None).unwrap(), 42_i32);
    }

    #[test]
    fn parse_float_with_property_no_property() {
        assert!((parse_float_with_property("3.14", None).unwrap() - 3.14_f32).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_byte_with_property_no_property() {
        assert_eq!(parse_byte_with_property("42", None).unwrap(), 42_i8);
    }

    #[test]
    fn parse_double_with_property_no_property() {
        assert!(
            (parse_double_with_property("3.14", None).unwrap() - 3.14_f64).abs() < f64::EPSILON
        );
    }

    // --- parse_big_decimal_with_property ---
    #[test]
    fn parse_big_decimal_with_property_no_property() {
        let val = parse_big_decimal_with_property("123.456", None).unwrap();
        assert_eq!(val, BigDecimal::from_str("123.456").unwrap());
    }
}
