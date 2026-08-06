//! 对应 Java：`com.alibaba.excel.converters.boolean_` 下的 Boolean 转换器
//!
//! 布尔值与数字/字符串单元格之间的双向转换辅助函数，
//! 供 `BooleanBooleanConverter` / `BooleanNumberConverter` / `BooleanStringConverter` 复用。

use bigdecimal::BigDecimal;
use num_bigint::BigInt;

use crate::{CellValue, ExcelError, ReadConverterContext, WriteCellData, WriteConverterContext};
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) trait BooleanScalar: Sized {
    fn from_boolean(value: bool) -> Self;
    fn is_one(&self) -> bool;
}

macro_rules! impl_boolean_scalar {
    ($($target:ty),+ $(,)?) => {
        $(
            #[allow(
                clippy::cast_precision_loss,
                clippy::float_cmp
                // 语义敏感：1/0 对所有 target 均精确可表示，且 Java `isOne()` 对
                // Float/Double 使用严格 `==`；保留 as 转换与 == 比较以 1:1 对应 Java 行为。
            )]
            impl BooleanScalar for $target {
                fn from_boolean(value: bool) -> Self {
                    if value { 1 as $target } else { 0 as $target }
                }

                fn is_one(&self) -> bool {
                    *self == 1 as $target
                }
            }
        )+
    };
}

impl_boolean_scalar!(i8, i16, i32, i64, f32, f64);

impl BooleanScalar for BigDecimal {
    fn from_boolean(value: bool) -> Self {
        Self::from(i32::from(value))
    }

    fn is_one(&self) -> bool {
        self == &Self::from(1)
    }
}

impl BooleanScalar for BigInt {
    fn from_boolean(value: bool) -> Self {
        Self::from(i32::from(value))
    }

    fn is_one(&self) -> bool {
        self == &Self::from(1)
    }
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn read_boolean_scalar<T>(context: &ReadConverterContext<'_>) -> Result<T, ExcelError>
where
    T: BooleanScalar,
{
    match context.cell() {
        Some(CellValue::Bool(value)) => Ok(T::from_boolean(*value)),
        Some(value) => Err(context.convert_context().invalid(value, "boolean scalar")),
        None => Err(context
            .convert_context()
            .invalid(&CellValue::Empty, "boolean scalar")),
    }
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn write_scalar_boolean<T>(context: &WriteConverterContext<'_, T>) -> WriteCellData
where
    T: BooleanScalar,
{
    WriteCellData::new(CellValue::Bool(context.value().is_one()))
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn read_boolean(context: &ReadConverterContext<'_>) -> Result<bool, ExcelError> {
    match context.cell() {
        Some(CellValue::Bool(value)) => Ok(*value),
        Some(value) => Err(context.convert_context().invalid(value, "bool")),
        None => Err(context.convert_context().invalid(&CellValue::Empty, "bool")),
    }
}

#[allow(clippy::float_cmp)]
// 语义敏感：对应 Java `BooleanNumberConverter` 对 Double 单元格的严格 `== 1.0`
// 判断，必须保留精确比较，不能用误差容忍替代。
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn read_number_boolean(context: &ReadConverterContext<'_>) -> Result<bool, ExcelError> {
    match context.cell() {
        Some(CellValue::Int(value)) => Ok(*value == 1),
        Some(CellValue::Float(value)) => Ok(*value == 1.0),
        Some(CellValue::Decimal(value)) => Ok(value == &BigDecimal::from(1)),
        Some(value) => Err(context.convert_context().invalid(value, "bool")),
        None => Err(context.convert_context().invalid(&CellValue::Empty, "bool")),
    }
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn write_boolean_number(context: &WriteConverterContext<'_, bool>) -> WriteCellData {
    WriteCellData::new(CellValue::Decimal(BigDecimal::from(i32::from(
        *context.value(),
    ))))
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn read_string_boolean(context: &ReadConverterContext<'_>) -> Result<bool, ExcelError> {
    match context.cell() {
        Some(CellValue::String(value)) => Ok(value.eq_ignore_ascii_case("true")),
        Some(value) => Err(context.convert_context().invalid(value, "bool")),
        None => Err(context.convert_context().invalid(&CellValue::Empty, "bool")),
    }
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn write_boolean_string(context: &WriteConverterContext<'_, bool>) -> WriteCellData {
    WriteCellData::from_string(context.value().to_string())
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn read_boolean_string_value(
    context: &ReadConverterContext<'_>,
) -> Result<String, ExcelError> {
    read_boolean(context).map(|value| value.to_string())
}
/// 对应 Java：com.alibaba.excel.converters.boolean_。
pub(crate) fn write_string_boolean(context: &WriteConverterContext<'_, String>) -> WriteCellData {
    WriteCellData::new(CellValue::Bool(
        context.value().eq_ignore_ascii_case("true"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converters::bigdecimal::big_decimal_boolean_converter::BigDecimalBooleanConverter;
    use crate::converters::biginteger::big_integer_boolean_converter::BigIntegerBooleanConverter;
    use crate::converters::booleanconverter::boolean_boolean_converter::BooleanBooleanConverter;
    use crate::converters::booleanconverter::boolean_number_converter::BooleanNumberConverter;
    use crate::converters::booleanconverter::boolean_string_converter::BooleanStringConverter;
    use crate::converters::byteconverter::byte_boolean_converter::ByteBooleanConverter;
    use crate::converters::doubleconverter::double_boolean_converter::DoubleBooleanConverter;
    use crate::converters::floatconverter::float_boolean_converter::FloatBooleanConverter;
    use crate::converters::integer::integer_boolean_converter::IntegerBooleanConverter;
    use crate::converters::longconverter::long_boolean_converter::LongBooleanConverter;
    use crate::converters::shortconverter::short_boolean_converter::ShortBooleanConverter;
    use crate::converters::string::string_boolean_converter::StringBooleanConverter;
    use crate::{
        CellDataType, ConvertContext, Converter, ExcelColumn, ReadConverterContext,
        WriteConverterContext,
    };

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
    fn scalar_boolean_converters_use_exact_java_zero_one_rules() {
        let context = context();
        let true_cell = CellValue::Bool(true);
        let read = ReadConverterContext::new(Some(&true_cell), &COLUMN, &context);
        assert_eq!(
            BigDecimalBooleanConverter.convert_to_rust_data(&read),
            Ok(BigDecimal::from(1))
        );
        assert_eq!(
            BigIntegerBooleanConverter.convert_to_rust_data(&read),
            Ok(BigInt::from(1))
        );
        assert_eq!(ByteBooleanConverter.convert_to_rust_data(&read), Ok(1));
        assert_eq!(ShortBooleanConverter.convert_to_rust_data(&read), Ok(1));
        assert_eq!(IntegerBooleanConverter.convert_to_rust_data(&read), Ok(1));
        assert_eq!(LongBooleanConverter.convert_to_rust_data(&read), Ok(1));
        assert_eq!(FloatBooleanConverter.convert_to_rust_data(&read), Ok(1.0));
        assert_eq!(DoubleBooleanConverter.convert_to_rust_data(&read), Ok(1.0));

        let one = 1_i32;
        let two = 2_i32;
        assert_eq!(
            IntegerBooleanConverter
                .convert_to_excel_data(&WriteConverterContext::new(&one, &COLUMN, &context))
                .unwrap()
                .value(),
            &CellValue::Bool(true)
        );
        assert_eq!(
            IntegerBooleanConverter
                .convert_to_excel_data(&WriteConverterContext::new(&two, &COLUMN, &context))
                .unwrap()
                .value(),
            &CellValue::Bool(false)
        );
    }

    #[test]
    fn boolean_number_and_string_converters_match_java_value_of() {
        let context = context();
        for (cell, expected) in [
            (CellValue::Int(1), true),
            (CellValue::Float(1.0), true),
            (CellValue::Decimal(BigDecimal::from(1)), true),
            (CellValue::Int(0), false),
            (CellValue::Int(2), false),
            (CellValue::Float(-1.0), false),
        ] {
            let read = ReadConverterContext::new(Some(&cell), &COLUMN, &context);
            assert_eq!(
                BooleanNumberConverter.convert_to_rust_data(&read),
                Ok(expected)
            );
        }

        for (text, expected) in [
            ("true", true),
            ("TrUe", true),
            ("false", false),
            ("1", false),
            (" true ", false),
            ("anything", false),
        ] {
            let cell = CellValue::String(text.to_owned());
            let read = ReadConverterContext::new(Some(&cell), &COLUMN, &context);
            assert_eq!(
                BooleanStringConverter.convert_to_rust_data(&read),
                Ok(expected)
            );
        }

        for value in [true, false] {
            let write = WriteConverterContext::new(&value, &COLUMN, &context);
            assert_eq!(
                BooleanBooleanConverter
                    .convert_to_excel_data(&write)
                    .unwrap()
                    .value(),
                &CellValue::Bool(value)
            );
            assert_eq!(
                BooleanStringConverter
                    .convert_to_excel_data(&write)
                    .unwrap()
                    .value(),
                &CellValue::String(value.to_string())
            );
            assert_eq!(
                BooleanNumberConverter
                    .convert_to_excel_data(&write)
                    .unwrap()
                    .value(),
                &CellValue::Decimal(BigDecimal::from(i32::from(value)))
            );
        }
    }

    #[test]
    fn string_boolean_converter_is_bidirectional_and_registered_by_boolean_source() {
        let context = context();
        let true_cell = CellValue::Bool(true);
        let read = ReadConverterContext::new(Some(&true_cell), &COLUMN, &context);
        assert_eq!(
            StringBooleanConverter.convert_to_rust_data(&read),
            Ok("true".to_owned())
        );
        for (text, expected) in [("TRUE", true), ("1", false), ("yes", false)] {
            let text = text.to_owned();
            let write = WriteConverterContext::new(&text, &COLUMN, &context);
            assert_eq!(
                StringBooleanConverter
                    .convert_to_excel_data(&write)
                    .unwrap()
                    .value(),
                &CellValue::Bool(expected)
            );
        }
        assert_eq!(
            <StringBooleanConverter as Converter<String>>::support_excel_type(
                &StringBooleanConverter
            ),
            CellDataType::Boolean
        );
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::converters::bigdecimal::big_decimal_boolean_converter::BigDecimalBooleanConverter;
    use crate::converters::biginteger::big_integer_boolean_converter::BigIntegerBooleanConverter;
    use crate::converters::booleanconverter::boolean_boolean_converter::BooleanBooleanConverter;
    use crate::converters::booleanconverter::boolean_number_converter::BooleanNumberConverter;
    use crate::converters::booleanconverter::boolean_string_converter::BooleanStringConverter;
    use crate::converters::byteconverter::byte_boolean_converter::ByteBooleanConverter;
    use crate::{
        ConvertContext, Converter, ExcelColumn, ReadConverterContext, WriteConverterContext,
    };

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
    fn big_decimal_and_big_int_scalar_booleans_write_one_and_zero() {
        // 对应 Java：`BigDecimal.ONE` / `BigInteger.ONE` 的 isOne 判断
        let context = context();
        let column = &COLUMN;
        let one_decimal = BigDecimal::from(1);
        let zero_decimal = BigDecimal::from(0);
        assert_eq!(
            BigDecimalBooleanConverter
                .convert_to_excel_data(&WriteConverterContext::new(&one_decimal, column, &context,))
                .unwrap()
                .value(),
            &CellValue::Bool(true)
        );
        assert_eq!(
            BigDecimalBooleanConverter
                .convert_to_excel_data(
                    &WriteConverterContext::new(&zero_decimal, column, &context,)
                )
                .unwrap()
                .value(),
            &CellValue::Bool(false)
        );
        let one_big_int = BigInt::from(1);
        let two_big_int = BigInt::from(2);
        assert_eq!(
            BigIntegerBooleanConverter
                .convert_to_excel_data(&WriteConverterContext::new(&one_big_int, column, &context,))
                .unwrap()
                .value(),
            &CellValue::Bool(true)
        );
        assert_eq!(
            BigIntegerBooleanConverter
                .convert_to_excel_data(&WriteConverterContext::new(&two_big_int, column, &context,))
                .unwrap()
                .value(),
            &CellValue::Bool(false)
        );
    }

    #[test]
    fn scalar_boolean_read_rejects_wrong_or_missing_cells() {
        // 对应 Java：布尔标量转换器对非布尔单元格或空单元格报错
        let context = context();
        let string_cell = CellValue::String("x".to_owned());
        let read = ReadConverterContext::new(Some(&string_cell), &COLUMN, &context);
        assert!(ByteBooleanConverter.convert_to_rust_data(&read).is_err());
        let empty_read = ReadConverterContext::new(None, &COLUMN, &context);
        assert!(
            ByteBooleanConverter
                .convert_to_rust_data(&empty_read)
                .is_err()
        );
    }

    #[test]
    fn boolean_converters_reject_wrong_or_missing_cells() {
        // 对应 Java：Boolean 转换器对非布尔单元格或空单元格报错
        let context = context();
        let int_cell = CellValue::Int(1);
        let read = ReadConverterContext::new(Some(&int_cell), &COLUMN, &context);
        assert!(BooleanBooleanConverter.convert_to_rust_data(&read).is_err());
        let empty_read = ReadConverterContext::new(None, &COLUMN, &context);
        assert!(
            BooleanBooleanConverter
                .convert_to_rust_data(&empty_read)
                .is_err()
        );

        let string_cell = CellValue::String("true".to_owned());
        let read = ReadConverterContext::new(Some(&string_cell), &COLUMN, &context);
        assert!(BooleanNumberConverter.convert_to_rust_data(&read).is_err());
        let empty_read = ReadConverterContext::new(None, &COLUMN, &context);
        assert!(
            BooleanNumberConverter
                .convert_to_rust_data(&empty_read)
                .is_err()
        );

        let int_cell = CellValue::Int(1);
        let read = ReadConverterContext::new(Some(&int_cell), &COLUMN, &context);
        assert!(BooleanStringConverter.convert_to_rust_data(&read).is_err());
        let empty_read = ReadConverterContext::new(None, &COLUMN, &context);
        assert!(
            BooleanStringConverter
                .convert_to_rust_data(&empty_read)
                .is_err()
        );
    }
}
