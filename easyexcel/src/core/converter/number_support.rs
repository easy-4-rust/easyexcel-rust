//! 对应 Java：`com.alibaba.excel.converters` 下的数字转换器与 `com.alibaba.excel.util.NumberUtils`
//!
//! Java 兼容的数字读写辅助：`JavaNumber` 抽象、数字与字符串单元格双向转换、
//! `BigDecimal`/`BigInt` 数值处理与 Java 二进制补码字节转换。

use std::str::FromStr;

use bigdecimal::{BigDecimal, ToPrimitive};
use num_bigint::{BigInt, Sign};

use crate::core::util::number_utils::{
    NonFiniteNumber, format_decimal, format_non_finite, parse_decimal,
};
use crate::core::util::work_book_util::fill_data_format;
use crate::{CellValue, ExcelError, ReadConverterContext, WriteCellData, WriteConverterContext};

pub(crate) trait JavaNumber: Sized {
    fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError>;
    fn to_decimal(&self) -> Result<BigDecimal, ExcelError>;
    fn java_string(&self) -> String;

    fn negative(&self) -> bool {
        false
    }

    fn non_finite(&self) -> Option<NonFiniteNumber> {
        None
    }
}

pub(crate) fn read_number<T>(context: &ReadConverterContext<'_>) -> Result<T, ExcelError>
where
    T: JavaNumber,
{
    let cell = context.cell().unwrap_or(&CellValue::Empty);
    let decimal = match cell {
        CellValue::Decimal(value) => value.clone(),
        CellValue::Int(value) => BigDecimal::from(*value),
        CellValue::Float(value) if value.is_finite() => BigDecimal::from_str(&value.to_string())
            .map_err(|_| context.convert_context().invalid(cell, "number"))?,
        other => return Err(context.convert_context().invalid(other, "number")),
    };
    T::from_decimal(&decimal).map_err(|error| number_error(context, cell, error))
}

pub(crate) fn write_number<T>(
    context: &WriteConverterContext<'_, T>,
) -> Result<WriteCellData, ExcelError>
where
    T: JavaNumber,
{
    let mut cell = WriteCellData::new(CellValue::Decimal(context.value().to_decimal()?));
    if let Some(format) = context
        .column()
        .format
        .or(context.convert_context().format)
        .filter(|format| !format.trim().is_empty())
    {
        fill_data_format(&mut cell, Some(format), "");
    }
    Ok(cell)
}

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
        context.column().format.or(context.convert_context().format),
    )
    .map_err(|_| context.convert_context().invalid(cell, "numeric string"))?;
    T::from_decimal(&decimal).map_err(|error| number_error(context, cell, error))
}

pub(crate) fn write_number_string<T>(
    context: &WriteConverterContext<'_, T>,
) -> Result<WriteCellData, ExcelError>
where
    T: JavaNumber,
{
    let pattern = context.column().format.or(context.convert_context().format);
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

fn decimal_to_big_int(value: &BigDecimal) -> BigInt {
    value.with_scale(0).into_bigint_and_exponent().0
}

fn java_signed_low_bytes<const N: usize>(value: &BigInt) -> [u8; N] {
    let sign_extension = if value.sign() == Sign::Minus {
        u8::MAX
    } else {
        0
    };
    let mut output = [sign_extension; N];
    let source = value.to_signed_bytes_le();
    let count = source.len().min(N);
    output[..count].copy_from_slice(&source[..count]);
    output
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
        Ok(decimal_to_big_int(value))
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
    ($target:ty, $bytes:expr) => {
        impl JavaNumber for $target {
            fn from_decimal(value: &BigDecimal) -> Result<Self, ExcelError> {
                let integer = decimal_to_big_int(value);
                Ok(<$target>::from_le_bytes(java_signed_low_bytes::<$bytes>(
                    &integer,
                )))
            }

            fn to_decimal(&self) -> Result<BigDecimal, ExcelError> {
                Ok(BigDecimal::from(*self))
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

impl_java_integer!(i8, 1);
impl_java_integer!(i16, 2);
impl_java_integer!(i32, 4);
impl_java_integer!(i64, 8);

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

    fn java_string(&self) -> String {
        java_f32_string(*self)
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

    fn java_string(&self) -> String {
        java_f64_string(*self)
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

fn java_f32_string(value: f32) -> String {
    java_float_string(
        f64::from(value),
        value.to_string(),
        &format!("{value:e}"),
        value.is_sign_negative(),
    )
}

fn java_f64_string(value: f64) -> String {
    java_float_string(
        value,
        value.to_string(),
        &format!("{value:e}"),
        value.is_sign_negative(),
    )
}

fn java_float_string(value: f64, plain: String, scientific: &str, negative: bool) -> String {
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }
    if value == 0.0 {
        return if negative { "-0.0" } else { "0.0" }.to_owned();
    }
    let absolute = value.abs();
    if !(1.0e-3..1.0e7).contains(&absolute) {
        let (mantissa, exponent) = scientific
            .split_once(['e', 'E'])
            .expect("Rust scientific formatting contains an exponent");
        let mantissa = if mantissa.contains('.') {
            mantissa.to_owned()
        } else {
            format!("{mantissa}.0")
        };
        return format!("{mantissa}E{}", exponent.trim_start_matches('+'));
    }
    if plain.contains('.') {
        plain
    } else {
        format!("{plain}.0")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::converter::bigdecimal::big_decimal_number_converter::BigDecimalNumberConverter;
    use crate::core::converter::bigdecimal::big_decimal_string_converter::BigDecimalStringConverter;
    use crate::core::converter::biginteger::big_integer_number_converter::BigIntegerNumberConverter;
    use crate::core::converter::biginteger::big_integer_string_converter::BigIntegerStringConverter;
    use crate::core::converter::byteconverter::byte_number_converter::ByteNumberConverter;
    use crate::core::converter::byteconverter::byte_string_converter::ByteStringConverter;
    use crate::core::converter::doubleconverter::double_number_converter::DoubleNumberConverter;
    use crate::core::converter::doubleconverter::double_string_converter::DoubleStringConverter;
    use crate::core::converter::floatconverter::float_number_converter::FloatNumberConverter;
    use crate::core::converter::floatconverter::float_string_converter::FloatStringConverter;
    use crate::core::converter::integer::integer_number_converter::IntegerNumberConverter;
    use crate::core::converter::integer::integer_string_converter::IntegerStringConverter;
    use crate::core::converter::longconverter::long_number_converter::LongNumberConverter;
    use crate::core::converter::longconverter::long_string_converter::LongStringConverter;
    use crate::core::converter::shortconverter::short_number_converter::ShortNumberConverter;
    use crate::core::converter::shortconverter::short_string_converter::ShortStringConverter;
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
    use crate::core::converter::integer::integer_string_converter::IntegerStringConverter;
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
}
