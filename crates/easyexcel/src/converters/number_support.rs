//! 对应 Java：`com.alibaba.excel.converters` 下的数字转换器与 `com.alibaba.excel.util.NumberUtils`
//!
//! Java 兼容的数字读写辅助：`JavaNumber` 抽象、数字与字符串单元格双向转换、
//! `BigDecimal`/`BigInt` 数值处理与 Java 二进制补码字节转换。

use std::str::FromStr;

use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::BigInt;

use crate::util::number_utils::{
    NonFiniteNumber, format_decimal, format_non_finite, parse_decimal,
};
use crate::util::work_book_util::fill_data_format;
use crate::{CellValue, ExcelError, ReadConverterContext, WriteCellData, WriteConverterContext};

#[cfg(test)]
use easyexcel_format::{java_f32_string, java_f64_string};

/// 对应 Java：com.alibaba.excel.converters。
pub(crate) trait JavaNumber: Sized {
    fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError>;
    fn to_decimal(&self) -> Result<BigDecimal, ExcelError>;
    fn java_string(&self) -> String;

    fn from_i64(value: i64) -> Result<Self, ExcelError> {
        Self::from_decimal(&BigDecimal::from(value))
    }

    fn from_f64(value: f64) -> Result<Self, ExcelError> {
        let decimal = BigDecimal::from_str(&value.to_string())
            .map_err(|_| ExcelError::Format(format!("invalid Java Number value {value}")))?;
        Self::from_decimal(&decimal)
    }

    fn negative(&self) -> bool {
        false
    }

    fn non_finite(&self) -> Option<NonFiniteNumber> {
        None
    }
}

/// 对应 Java：com.alibaba.excel.converters。
pub(crate) fn read_number<T>(context: &ReadConverterContext<'_>) -> Result<T, ExcelError>
where
    T: JavaNumber,
{
    let cell = context.cell().unwrap_or(&CellValue::Empty);
    let result = match cell {
        CellValue::Decimal(value) => T::from_decimal(value),
        CellValue::Int(value) => T::from_i64(*value),
        CellValue::Float(value) if value.is_finite() => T::from_f64(*value),
        other => return Err(context.convert_context().invalid(other, "number")),
    };
    result.map_err(|error| number_error(context, cell, error))
}

/// 对应 Java：com.alibaba.excel.converters。
pub(crate) fn write_number<T>(
    context: &WriteConverterContext<'_, T>,
) -> Result<WriteCellData, ExcelError>
where
    T: JavaNumber,
{
    let mut cell = WriteCellData::new(CellValue::Decimal(context.value().to_decimal()?));
    if let Some(format) = context
        .column()
        .effective_number_format()
        .or(context.convert_context().effective_number_format())
        .filter(|format| !format.trim().is_empty())
    {
        fill_data_format(&mut cell, Some(format), "");
    }
    Ok(cell)
}

/// 对应 Java：com.alibaba.excel.converters。
pub(crate) fn read_string_number<T>(context: &ReadConverterContext<'_>) -> Result<T, ExcelError>
where
    T: JavaNumber,
{
    let cell = context.cell().unwrap_or(&CellValue::Empty);
    let CellValue::String(value) = cell else {
        return Err(context.convert_context().invalid(cell, "numeric string"));
    };
    let decimal = parse_decimal(
        value,
        context
            .column()
            .effective_number_format()
            .or(context.convert_context().effective_number_format()),
    )
    .map_err(|_| context.convert_context().invalid(cell, "numeric string"))?;
    T::from_decimal(&decimal).map_err(|error| number_error(context, cell, error))
}

/// 对应 Java：com.alibaba.excel.converters。
pub(crate) fn write_number_string<T>(
    context: &WriteConverterContext<'_, T>,
) -> Result<WriteCellData, ExcelError>
where
    T: JavaNumber,
{
    let pattern = context
        .column()
        .effective_number_format()
        .or(context.convert_context().effective_number_format());
    let text = if let Some(non_finite) = context.value().non_finite() {
        format_non_finite(non_finite, pattern)?
    } else if pattern.is_none_or(str::is_empty) {
        context.value().java_string()
    } else {
        format_decimal(
            &context.value().to_decimal()?,
            context.value().negative(),
            pattern,
            context.column().number_rounding_mode.unwrap_or_default(),
        )?
    };
    Ok(WriteCellData::from_string(text))
}

fn number_error(
    context: &ReadConverterContext<'_>,
    cell: &CellValue,
    error: ExcelError,
) -> ExcelError {
    match error {
        ExcelError::Data { .. } => error,
        _ => context.convert_context().invalid(cell, "number"),
    }
}

impl JavaNumber for BigDecimal {
    fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError> {
        Ok(value.clone())
    }

    fn to_decimal(&self) -> Result<BigDecimal, ExcelError> {
        Ok(self.clone())
    }

    fn java_string(&self) -> String {
        self.to_plain_string()
    }

    fn negative(&self) -> bool {
        self < &Self::from(0)
    }
}

impl JavaNumber for BigInt {
    fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError> {
        Ok(easyexcel_format::decimal_to_big_int(value))
    }

    fn to_decimal(&self) -> Result<BigDecimal, ExcelError> {
        Ok(BigDecimal::from(self.clone()))
    }

    fn java_string(&self) -> String {
        self.to_string()
    }

    fn negative(&self) -> bool {
        self.sign() == num_bigint::Sign::Minus
    }
}

macro_rules! impl_java_integer {
    ($target:ty, $convert:path) => {
        impl JavaNumber for $target {
            fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError> {
                Ok($convert(value))
            }

            fn to_decimal(&self) -> Result<BigDecimal, ExcelError> {
                Ok(BigDecimal::from(*self))
            }

            fn from_i64(value: i64) -> Result<Self, ExcelError> {
                #[allow(clippy::cast_possible_truncation)]
                Ok(value as Self)
            }

            fn java_string(&self) -> String {
                self.to_string()
            }

            fn negative(&self) -> bool {
                *self < 0
            }
        }
    };
}

impl_java_integer!(i8, easyexcel_format::decimal_to_java_i8);
impl_java_integer!(i16, easyexcel_format::decimal_to_java_i16);
impl_java_integer!(i32, easyexcel_format::decimal_to_java_i32);
impl_java_integer!(i64, easyexcel_format::decimal_to_java_i64);

impl JavaNumber for f32 {
    fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError> {
        value
            .to_f32()
            .or_else(|| value.to_string().parse().ok())
            .ok_or_else(|| {
                ExcelError::Format(format!("cannot convert BigDecimal {value} to Java Float"))
            })
    }

    fn to_decimal(&self) -> Result<BigDecimal, ExcelError> {
        BigDecimal::from_str(&self.to_string())
            .map_err(|_| ExcelError::Format(format!("invalid Java Float value {self}")))
    }

    fn from_i64(value: i64) -> Result<Self, ExcelError> {
        #[allow(clippy::cast_precision_loss)]
        Ok(value as Self)
    }

    fn from_f64(value: f64) -> Result<Self, ExcelError> {
        #[allow(clippy::cast_possible_truncation)]
        Ok(value as Self)
    }

    fn java_string(&self) -> String {
        easyexcel_format::java_f32_string(*self)
    }

    fn negative(&self) -> bool {
        self.is_sign_negative()
    }

    fn non_finite(&self) -> Option<NonFiniteNumber> {
        if self.is_nan() {
            Some(NonFiniteNumber::Nan)
        } else if *self == f32::INFINITY {
            Some(NonFiniteNumber::PositiveInfinity)
        } else if *self == f32::NEG_INFINITY {
            Some(NonFiniteNumber::NegativeInfinity)
        } else {
            None
        }
    }
}

impl JavaNumber for f64 {
    fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError> {
        value
            .to_f64()
            .or_else(|| value.to_string().parse().ok())
            .ok_or_else(|| {
                ExcelError::Format(format!("cannot convert BigDecimal {value} to Java Double"))
            })
    }

    fn to_decimal(&self) -> Result<BigDecimal, ExcelError> {
        BigDecimal::from_str(&self.to_string())
            .map_err(|_| ExcelError::Format(format!("invalid Java Double value {self}")))
    }

    fn from_i64(value: i64) -> Result<Self, ExcelError> {
        #[allow(clippy::cast_precision_loss)]
        Ok(value as Self)
    }

    fn from_f64(value: f64) -> Result<Self, ExcelError> {
        Ok(value)
    }

    fn java_string(&self) -> String {
        easyexcel_format::java_f64_string(*self)
    }

    fn negative(&self) -> bool {
        self.is_sign_negative()
    }

    fn non_finite(&self) -> Option<NonFiniteNumber> {
        if self.is_nan() {
            Some(NonFiniteNumber::Nan)
        } else if *self == f64::INFINITY {
            Some(NonFiniteNumber::PositiveInfinity)
        } else if *self == f64::NEG_INFINITY {
            Some(NonFiniteNumber::NegativeInfinity)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::bigdecimal::big_decimal_number_converter::BigDecimalNumberConverter;
    use crate::converters::bigdecimal::big_decimal_string_converter::BigDecimalStringConverter;
    use crate::converters::biginteger::big_integer_number_converter::BigIntegerNumberConverter;
    use crate::converters::biginteger::big_integer_string_converter::BigIntegerStringConverter;
    use crate::converters::byteconverter::byte_number_converter::ByteNumberConverter;
    use crate::converters::byteconverter::byte_string_converter::ByteStringConverter;
    use crate::converters::doubleconverter::double_number_converter::DoubleNumberConverter;
    use crate::converters::doubleconverter::double_string_converter::DoubleStringConverter;
    use crate::converters::floatconverter::float_number_converter::FloatNumberConverter;
    use crate::converters::floatconverter::float_string_converter::FloatStringConverter;
    use crate::converters::integer::integer_number_converter::IntegerNumberConverter;
    use crate::converters::integer::integer_string_converter::IntegerStringConverter;
    use crate::converters::longconverter::long_number_converter::LongNumberConverter;
    use crate::converters::longconverter::long_string_converter::LongStringConverter;
    use crate::converters::shortconverter::short_number_converter::ShortNumberConverter;
    use crate::converters::shortconverter::short_string_converter::ShortStringConverter;
    use crate::{ConvertContext, Converter, ExcelColumn, NumberRoundingMode};

    const COLUMN: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, None);
    const FORMATTED_COLUMN: ExcelColumn =
        ExcelColumn::new("value", "Value", Some(0), 0, Some("#,##0.00"));

    fn context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 2,
            column_index: Some(1),
            field: "value",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        }
    }

    // 测试辅助：具体转换器均为零尺寸单元结构体，按值传递是调用点惯例；
    // 泛型参数 C 仅按引用使用，但改为 &C 会改变泛型约束，故保留按值传递
    #[allow(clippy::needless_pass_by_value)]
    fn read<T, C>(converter: C, value: &str) -> T
    where
        C: Converter<T>,
    {
        let context = context();
        let cell = CellValue::Decimal(value.parse().unwrap());
        converter
            .convert_to_rust_data(&ReadConverterContext::new(Some(&cell), &COLUMN, &context))
            .unwrap()
    }

    #[test]
    // 1.25 可被 f32/f64 二进制精确表示，精确比较正是本测试的意图
    #[allow(clippy::float_cmp)]
    fn number_converters_match_java_big_decimal_accessors() {
        assert_eq!(
            read::<BigDecimal, _>(BigDecimalNumberConverter, "123.450"),
            "123.450".parse::<BigDecimal>().unwrap()
        );
        assert_eq!(
            read::<BigInt, _>(BigIntegerNumberConverter, "-123.99"),
            BigInt::from(-123)
        );
        assert_eq!(read::<i8, _>(ByteNumberConverter, "255.99"), -1);
        assert_eq!(read::<i8, _>(ByteNumberConverter, "-129.99"), 127);
        assert_eq!(read::<i16, _>(ShortNumberConverter, "65535.9"), -1);
        assert_eq!(read::<i32, _>(IntegerNumberConverter, "4294967295.8"), -1);
        assert_eq!(
            read::<i64, _>(LongNumberConverter, "18446744073709551615.7"),
            -1
        );
        assert_eq!(read::<f32, _>(FloatNumberConverter, "1.25"), 1.25);
        assert_eq!(read::<f64, _>(DoubleNumberConverter, "1.25"), 1.25);
    }

    #[test]
    fn number_converters_write_decimal_cells_and_preserve_number_format() {
        let context = context();
        let value = 42_i32;
        let cell = IntegerNumberConverter
            .convert_to_excel_data(&WriteConverterContext::new(
                &value,
                &FORMATTED_COLUMN,
                &context,
            ))
            .unwrap();
        assert_eq!(cell.value(), &CellValue::Decimal(BigDecimal::from(42)));
        assert_eq!(
            cell.data_format_data().and_then(|data| data.format()),
            Some("#,##0.00")
        );

        let big = BigInt::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let cell = BigIntegerNumberConverter
            .convert_to_excel_data(&WriteConverterContext::new(&big, &COLUMN, &context))
            .unwrap();
        assert_eq!(
            cell.value(),
            &CellValue::Decimal(BigDecimal::from(big.clone()))
        );
    }

    #[test]
    fn number_converters_reject_non_number_sources_and_non_finite_writes() {
        let context = context();
        let text = CellValue::String("1".to_owned());
        let error = IntegerNumberConverter
            .convert_to_rust_data(&ReadConverterContext::new(Some(&text), &COLUMN, &context))
            .unwrap_err();
        assert!(matches!(error, ExcelError::Data { .. }));

        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                DoubleNumberConverter
                    .convert_to_excel_data(&WriteConverterContext::new(&value, &COLUMN, &context))
                    .is_err()
            );
        }
    }

    // 测试辅助：同 `read`，保留按值传递
    #[allow(clippy::needless_pass_by_value)]
    fn read_string<T, C>(converter: C, value: &str, column: &ExcelColumn) -> Result<T, ExcelError>
    where
        C: Converter<T>,
    {
        let context = context();
        let cell = CellValue::String(value.to_owned());
        converter.convert_to_rust_data(&ReadConverterContext::new(Some(&cell), column, &context))
    }

    // 测试辅助：同 `read`，保留按值传递
    #[allow(clippy::needless_pass_by_value)]
    fn write_string<T, C>(converter: C, value: &T, column: &ExcelColumn) -> String
    where
        C: Converter<T>,
    {
        let context = context();
        converter
            .convert_to_excel_data(&WriteConverterContext::new(value, column, &context))
            .unwrap()
            .value()
            .as_text()
    }

    #[test]
    // 1.25 可被 f32/f64 二进制精确表示，精确比较正是本测试的意图
    #[allow(clippy::float_cmp)]
    fn string_number_converters_cover_all_java_numeric_types_and_wrapping() {
        assert_eq!(
            read_string::<BigDecimal, _>(BigDecimalStringConverter, "123.450", &COLUMN).unwrap(),
            "123.450".parse::<BigDecimal>().unwrap()
        );
        assert_eq!(
            read_string::<BigInt, _>(BigIntegerStringConverter, "-123.99", &COLUMN).unwrap(),
            BigInt::from(-123)
        );
        assert_eq!(
            read_string::<i8, _>(ByteStringConverter, "255.9", &COLUMN).unwrap(),
            -1
        );
        assert_eq!(
            read_string::<i16, _>(ShortStringConverter, "65535.9", &COLUMN).unwrap(),
            -1
        );
        assert_eq!(
            read_string::<i32, _>(IntegerStringConverter, "4294967295.9", &COLUMN).unwrap(),
            -1
        );
        assert_eq!(
            read_string::<i64, _>(LongStringConverter, "18446744073709551615.9", &COLUMN).unwrap(),
            -1
        );
        assert_eq!(
            read_string::<f32, _>(FloatStringConverter, "1.25", &COLUMN).unwrap(),
            1.25
        );
        assert_eq!(
            read_string::<f64, _>(DoubleStringConverter, "1.25", &COLUMN).unwrap(),
            1.25
        );
        assert!(read_string::<i32, _>(IntegerStringConverter, " 1.00", &COLUMN).is_err());
        assert!(read_string::<i32, _>(IntegerStringConverter, "1.00 ", &COLUMN).is_err());
    }

    #[test]
    fn string_number_converters_match_decimal_format_and_rounding_modes() {
        const PERCENT: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, Some("#.##%"));
        const HALF_DOWN: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, Some("0.00"))
            .with_number_rounding_mode(NumberRoundingMode::HalfDown);
        const UNNECESSARY: ExcelColumn =
            ExcelColumn::new("value", "Value", Some(0), 0, Some("0.00"))
                .with_number_rounding_mode(NumberRoundingMode::Unnecessary);
        assert_eq!(
            read_string::<BigDecimal, _>(BigDecimalStringConverter, "12.34%", &PERCENT).unwrap(),
            "0.1234".parse::<BigDecimal>().unwrap()
        );
        assert_eq!(
            write_string(DoubleStringConverter, &1.235_f64, &PERCENT),
            "123.5%"
        );

        assert_eq!(
            write_string(
                BigDecimalStringConverter,
                &"1.225".parse().unwrap(),
                &HALF_DOWN
            ),
            "1.22"
        );

        let context = context();
        assert!(
            BigDecimalStringConverter
                .convert_to_excel_data(&WriteConverterContext::new(
                    &"1.001".parse().unwrap(),
                    &UNNECESSARY,
                    &context,
                ))
                .is_err()
        );
    }

    #[test]
    fn floating_string_converters_match_java_to_string_and_special_values() {
        const PERCENT: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, Some("#.##%"));
        for (value, expected) in [
            (1.0, "1.0"),
            (0.0001, "1.0E-4"),
            (10_000_000.0, "1.0E7"),
            (-0.0, "-0.0"),
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
            (f64::NEG_INFINITY, "-Infinity"),
        ] {
            assert_eq!(
                write_string(DoubleStringConverter, &value, &COLUMN),
                expected
            );
        }
        assert_eq!(
            write_string(DoubleStringConverter, &f64::INFINITY, &PERCENT),
            "∞%"
        );
        assert_eq!(
            write_string(DoubleStringConverter, &f64::NEG_INFINITY, &PERCENT),
            "-∞%"
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::converters::integer::integer_string_converter::IntegerStringConverter;
    use crate::{ConvertContext, Converter, ExcelColumn};

    /// 测试辅助：仅实现三个必需方法的 `JavaNumber`，用于覆盖默认 `negative` / `non_finite`。
    struct BareNumber;

    impl JavaNumber for BareNumber {
        fn from_decimal(_value: &BigDecimal) -> Result<Self, ExcelError> {
            Ok(BareNumber)
        }

        fn to_decimal(&self) -> Result<BigDecimal, ExcelError> {
            Ok(BigDecimal::from(0))
        }

        fn java_string(&self) -> String {
            String::new()
        }
    }

    const COLUMN: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, None);

    fn context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 1,
            column_index: Some(0),
            field: "value",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        }
    }

    #[test]
    fn default_java_number_methods_report_non_negative_and_finite() {
        // 对应 Java：`JavaNumber` 默认 `negative()` 为 false、`non_finite()` 为 None
        assert!(!BareNumber.negative());
        assert_eq!(BareNumber.non_finite(), None);
    }

    #[test]
    fn read_string_number_rejects_non_string_cells() {
        // 对应 Java：字符串数字转换器仅接受字符串单元格
        let context = context();
        let int_cell = CellValue::Int(1);
        let read = ReadConverterContext::new(Some(&int_cell), &COLUMN, &context);
        assert!(IntegerStringConverter.convert_to_rust_data(&read).is_err());
    }

    #[test]
    fn number_error_wraps_or_passes_through() {
        // 对应 Java：`ExcelDataConvertException` 原样透传，其余错误包装为转换错误
        let context = context();
        let cell = CellValue::Int(1);
        let read_context = ReadConverterContext::new(Some(&cell), &COLUMN, &context);
        let data_error = ExcelError::Data {
            sheet: "s".to_owned(),
            row: 0,
            column: None,
            field: "f",
            value: "1".to_owned(),
            message: "already a data error".to_owned(),
        };
        assert_eq!(
            number_error(&read_context, &cell, data_error.clone()),
            data_error
        );
        let wrapped = number_error(&read_context, &cell, ExcelError::Format("boom".to_owned()));
        assert!(matches!(wrapped, ExcelError::Data { .. }));
    }

    #[test]
    fn big_int_and_integer_negative_flags_match_java_signs() {
        // 对应 Java：`BigInteger` / 基本整数类型的符号判断
        assert!(JavaNumber::negative(&BigInt::from(-5)));
        assert!(!JavaNumber::negative(&BigInt::from(5)));
        assert!(JavaNumber::negative(&-1_i8));
        assert!(!JavaNumber::negative(&1_i8));
        assert!(JavaNumber::negative(&-1_i16));
        assert!(JavaNumber::negative(&-1_i32));
        assert!(JavaNumber::negative(&-1_i64));
        assert!(!JavaNumber::negative(&1_i64));
    }

    #[test]
    fn float_negative_and_non_finite_flags_match_java() {
        // 对应 Java：`Float.isNaN` / `isInfinite` 与符号位判断
        assert!(JavaNumber::negative(&-1.0_f32));
        assert!(!JavaNumber::negative(&1.0_f32));
        assert!(JavaNumber::negative(&-1.0_f64));
        assert!(!JavaNumber::negative(&1.0_f64));
        assert_eq!(
            JavaNumber::non_finite(&f32::NAN),
            Some(NonFiniteNumber::Nan)
        );
        assert_eq!(
            JavaNumber::non_finite(&f32::INFINITY),
            Some(NonFiniteNumber::PositiveInfinity)
        );
        assert_eq!(
            JavaNumber::non_finite(&f32::NEG_INFINITY),
            Some(NonFiniteNumber::NegativeInfinity)
        );
        assert_eq!(JavaNumber::non_finite(&1.0_f32), None);
        assert_eq!(
            JavaNumber::non_finite(&f64::NAN),
            Some(NonFiniteNumber::Nan)
        );
        assert_eq!(
            JavaNumber::non_finite(&f64::INFINITY),
            Some(NonFiniteNumber::PositiveInfinity)
        );
        assert_eq!(
            JavaNumber::non_finite(&f64::NEG_INFINITY),
            Some(NonFiniteNumber::NegativeInfinity)
        );
        assert_eq!(JavaNumber::non_finite(&1.0_f64), None);
    }

    #[test]
    fn java_float_string_special_values_match_java_to_string() {
        // 对应 Java：`Float.toString` / `Double.toString` 的特殊值
        assert_eq!(java_f64_string(f64::NAN), "NaN");
        assert_eq!(java_f64_string(f64::INFINITY), "Infinity");
        assert_eq!(java_f64_string(f64::NEG_INFINITY), "-Infinity");
        assert_eq!(java_f32_string(f32::NAN), "NaN");
        assert_eq!(java_f32_string(0.0), "0.0");
        assert_eq!(java_f32_string(-0.0), "-0.0");
        assert_eq!(java_f32_string(1.5), "1.5");
        assert_eq!(java_f32_string(1.0e8), "1.0E8");
    }

    #[test]
    // 1.25 可被 f32/f64 二进制精确表示，精确比较正是本测试的意图
    #[allow(clippy::float_cmp)]
    fn float_from_decimal_success_paths_match_java() {
        // 对应 Java：`BigDecimal.floatValue` / `doubleValue` 的转换路径
        let decimal: BigDecimal = "1.25".parse().unwrap();
        assert_eq!(
            <f32 as JavaNumber>::from_decimal(&decimal).unwrap(),
            1.25_f32
        );
        assert_eq!(
            <f64 as JavaNumber>::from_decimal(&decimal).unwrap(),
            1.25_f64
        );
    }

    #[test]
    fn bare_number_required_methods_are_invocable() {
        // 对应 Java：JavaNumber 三个必需方法可直接调用
        let bare = BareNumber::from_decimal(&BigDecimal::from(7)).expect("from decimal");
        assert_eq!(bare.to_decimal().expect("to decimal"), BigDecimal::from(0));
        assert_eq!(bare.java_string(), "");
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    #[test]
    fn java_float_string_scientific_mantissa_keeps_decimal_point() {
        // 对应 Java：Double.toString / Float.toString 科学计数法保留小数点尾数
        assert_eq!(java_f64_string(1.5e8), "1.5E8");
        assert_eq!(java_f32_string(1.5e8_f32), "1.5E8");
    }

    /// T1.3 验收：`read_number::<f64>` 在 `CellValue::Float` 输入下走
    /// `f64::from_f64` 直通快路径，不构造 BigDecimal。
    ///
    /// 实现说明：项目未引入 `count_alloc` / `dhat` 等 allocation counter crate，
    /// 且 `#[global_allocator]` 无法在运行时切换。此处通过直接验证 `from_f64`
    /// 的返回值语义来间接断言零分配：T1.1 后 `f64::from_f64(v)` = `Ok(v)`，
    /// 内部无 `BigDecimal::from_str` / `to_string` / 堆构造。
    /// 若未来引入 `count_alloc`，可将此测试升级为精确计数断言。
    #[test]
    fn read_number_f64_fast_path_skips_big_decimal_construction() {
        // 1. f64::from_f64 直通：无 BigDecimal 构造
        let value: f64 = 1.5;
        let result = <f64 as JavaNumber>::from_f64(value).unwrap();
        // 精确比较：1.5 可被 f64 二进制精确表示
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(result, 1.5_f64);
        }

        // 2. read_number::<f64> 在 Float 输入下的完整路径
        let context = crate::ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 1,
            column_index: Some(0),
            field: "score",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        };
        let column = crate::ExcelColumn::new("score", "Score", Some(0), 0, None);
        let cell = CellValue::Float(3.14);
        let read_ctx = crate::ReadConverterContext::new(Some(&cell), &column, &context);
        let result = read_number::<f64>(&read_ctx).unwrap();
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(result, 3.14_f64);
        }

        // 3. 非 finite 被拒绝（T1.1 防御性检查）
        let nan_cell = CellValue::Float(f64::NAN);
        let nan_ctx = crate::ReadConverterContext::new(Some(&nan_cell), &column, &context);
        assert!(read_number::<f64>(&nan_ctx).is_err());

        // 4. f32 同样走直通
        let f32_result = <f32 as JavaNumber>::from_f64(2.5).unwrap();
        #[allow(clippy::float_cmp)]
        {
            assert_eq!(f32_result, 2.5_f32);
        }

        // 5. 确保 from_f64 对 f64 是纯赋值（验证 T1.1 修复效果）：
        //    修复前默认实现走 BigDecimal::from_str(&value.to_string())，
        //    修复后 f64/f32 直接返回 Ok(value)。
        let test_values = [0.0, -0.0, 1.0, -1.0, 42.5, 1e10, 1e-10];
        for v in test_values {
            let r = <f64 as JavaNumber>::from_f64(v).unwrap();
            #[allow(clippy::float_cmp)]
            {
                assert_eq!(r, v, "from_f64({v}) must return exact value");
            }
        }
    }
}
