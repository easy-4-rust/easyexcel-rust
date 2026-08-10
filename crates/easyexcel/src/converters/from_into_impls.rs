//! Mirrors the union of `com.alibaba.excel.converters.*.java` (the ~40
//! built-in `Converter<T>` implementations registered by Java's
//! `DefaultConverterLoader`).
//!
//! Each `impl FromExcelCell for X` and `impl IntoExcelCell for X` here
//! corresponds to a Java converter under
//! `com.alibaba.excel.converters.{bigdecimal,biginteger,booleanconverter,
//!  byteconverter,date,doubleconverter,floatconverter,integer,localdate,
//!  localdatetime,longconverter,shortconverter,string}` plus the
//! `Vec<u8>` / `Box<[u8]>` / `[u8; N]` / `PathBuf` image converters.

use std::fmt::Display;
use std::str::FromStr;

use bigdecimal::BigDecimal;
use bigdecimal::ToPrimitive;
use chrono::{NaiveDate, NaiveDateTime};
use num_bigint::BigInt;

use crate::core::cell_value::CellValue;
use crate::core::convert_context::ConvertContext;
use crate::core::dynamic_row::DynamicRow;
use crate::core::excel_error::ExcelError;
use crate::core::excel_row::ExcelRow;
use crate::core::from_excel_cell::FromExcelCell;
use crate::core::into_excel_cell::IntoExcelCell;
use crate::core::row_data::RowData;

impl FromExcelCell for String {
    fn from_excel_cell(
        value: Option<&CellValue>,
        _context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        Ok(value.map_or_else(String::new, CellValue::as_text))
    }
}

impl IntoExcelCell for String {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::String(self.clone()))
    }
}

impl IntoExcelCell for &str {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::String((*self).to_owned()))
    }
}

impl FromExcelCell for bool {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        match value.unwrap_or(&CellValue::Empty) {
            CellValue::Bool(value) => Ok(*value),
            CellValue::Int(value) => Ok(*value != 0),
            CellValue::Float(value) => Ok(*value != 0.0),
            CellValue::Decimal(value) => Ok(value != &BigDecimal::from(0)),
            CellValue::String(value) if value.eq_ignore_ascii_case("true") || value == "1" => {
                Ok(true)
            }
            CellValue::String(value) if value.eq_ignore_ascii_case("false") || value == "0" => {
                Ok(false)
            }
            other => Err(context.invalid(other, "bool")),
        }
    }
}

impl IntoExcelCell for bool {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::Bool(*self))
    }
}

macro_rules! integer_conversion {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl FromExcelCell for $ty {
                fn from_excel_cell(
                    value: Option<&CellValue>,
                    context: &ConvertContext,
                ) -> Result<Self, ExcelError> {
                    parse_integer(value, context, stringify!($ty))
                }
            }

            impl IntoExcelCell for $ty {
                fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
                    Ok(integer_to_cell(*self))
                }
            }
        )+
    };
}

integer_conversion!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl FromExcelCell for BigInt {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        let cell = value.unwrap_or(&CellValue::Empty);
        match cell {
            CellValue::Bool(value) => Ok(Self::from(u8::from(*value))),
            CellValue::Int(value) => Ok(Self::from(*value)),
            CellValue::Float(value) => BigDecimal::from_str(&value.to_string())
                .map(|value| easyexcel_format::decimal_to_big_int(&value))
                .map_err(|_| context.invalid(cell, "BigInt")),
            CellValue::Decimal(value) => Ok(easyexcel_format::decimal_to_big_int(value)),
            CellValue::String(value) => BigDecimal::from_str(value)
                .map(|value| easyexcel_format::decimal_to_big_int(&value))
                .map_err(|_| context.invalid(cell, "BigInt")),
            other => Err(context.invalid(other, "BigInt")),
        }
    }
}

impl IntoExcelCell for BigInt {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(self
            .to_i64()
            .map_or_else(|| CellValue::String(self.to_string()), CellValue::Int))
    }
}

fn parse_integer<T>(
    value: Option<&CellValue>,
    context: &ConvertContext,
    target: &'static str,
) -> Result<T, ExcelError>
where
    T: FromStr,
{
    let value = value.unwrap_or(&CellValue::Empty);
    let text = match value {
        CellValue::Bool(inner) => u8::from(*inner).to_string(),
        CellValue::Int(inner) => inner.to_string(),
        CellValue::Float(inner) if inner.fract() == 0.0 => inner.to_string(),
        CellValue::Decimal(inner) if inner == &inner.with_scale(0) => inner.to_string(),
        CellValue::String(inner) => inner.clone(),
        other => return Err(context.invalid(other, target)),
    };
    text.parse::<T>()
        .map_err(|_| context.invalid(value, target))
}

fn integer_to_cell<T>(value: T) -> CellValue
where
    T: TryInto<i64> + Display + Copy,
{
    value
        .try_into()
        .map_or_else(|_| CellValue::String(value.to_string()), CellValue::Int)
}

macro_rules! float_conversion {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl FromExcelCell for $ty {
                fn from_excel_cell(
                    value: Option<&CellValue>,
                    context: &ConvertContext,
                ) -> Result<Self, ExcelError> {
                    // 热路径：Float/Int 直接数值转换，跳过 String 往返。
                    // 对应 Java DoubleConverter/FloatConverter 对 numeric cell 的直读语义。
                    let cell = value.unwrap_or(&CellValue::Empty);
                    match cell {
                        CellValue::Float(inner) => {
                            // f64→Self（f64 恒等，f32 截断）；保持 non_finite 一致性。
                            Ok(*inner as Self)
                        }
                        CellValue::Int(inner) => Ok(*inner as Self),
                        // 非数值变体仍走 String 解析，保留 Java 兼容的格式语义。
                        _ => parse_float(value, context, stringify!($ty)),
                    }
                }
            }

            impl IntoExcelCell for $ty {
                fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
                    Ok(CellValue::Float(f64::from(*self)))
                }
            }
        )+
    };
}

float_conversion!(f32, f64);

fn parse_float<T>(
    value: Option<&CellValue>,
    context: &ConvertContext,
    target: &'static str,
) -> Result<T, ExcelError>
where
    T: FromStr,
{
    let value = value.unwrap_or(&CellValue::Empty);
    let text = match value {
        CellValue::Bool(inner) => u8::from(*inner).to_string(),
        CellValue::Int(inner) => inner.to_string(),
        CellValue::Float(inner) => inner.to_string(),
        CellValue::Decimal(inner) => inner.to_string(),
        CellValue::String(inner) => inner.clone(),
        other => return Err(context.invalid(other, target)),
    };
    text.parse::<T>()
        .map_err(|_| context.invalid(value, target))
}

impl FromExcelCell for BigDecimal {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        let value = value.unwrap_or(&CellValue::Empty);
        match value {
            CellValue::Bool(inner) => Ok(Self::from(u8::from(*inner))),
            CellValue::Decimal(inner) => Ok(inner.clone()),
            CellValue::Int(inner) => Ok(Self::from(*inner)),
            CellValue::Float(inner) => {
                Self::from_str(&inner.to_string()).map_err(|_| context.invalid(value, "BigDecimal"))
            }
            CellValue::String(inner) => {
                Self::from_str(inner).map_err(|_| context.invalid(value, "BigDecimal"))
            }
            other => Err(context.invalid(other, "BigDecimal")),
        }
    }
}

impl IntoExcelCell for BigDecimal {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::Decimal(self.clone()))
    }
}

impl FromExcelCell for NaiveDate {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        let value = value.unwrap_or(&CellValue::Empty);
        match value {
            CellValue::Date(value) => Ok(*value),
            CellValue::DateTime(value) => Ok(value.date()),
            CellValue::Int(_) | CellValue::Float(_) | CellValue::Decimal(_) => {
                excel_serial_to_datetime(value, context).map(|value| value.date())
            }
            CellValue::String(inner) => {
                let format = easyexcel_model::chrono_date_format(
                    context.effective_date_time_format().unwrap_or("%Y-%m-%d"),
                );
                NaiveDate::parse_from_str(inner, format.as_ref())
                    .map_err(|_| context.invalid(value, "NaiveDate"))
            }
            other => Err(context.invalid(other, "NaiveDate")),
        }
    }
}

impl IntoExcelCell for NaiveDate {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::Date(*self))
    }
}

impl FromExcelCell for NaiveDateTime {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        let value = value.unwrap_or(&CellValue::Empty);
        match value {
            CellValue::DateTime(value) => Ok(*value),
            CellValue::Date(value) => Ok(value.and_hms_opt(0, 0, 0).expect("midnight is valid")),
            CellValue::Int(_) | CellValue::Float(_) | CellValue::Decimal(_) => {
                excel_serial_to_datetime(value, context)
            }
            CellValue::String(inner) => {
                let format = easyexcel_model::chrono_date_format(
                    context
                        .effective_date_time_format()
                        .unwrap_or("%Y-%m-%d %H:%M:%S"),
                );
                NaiveDateTime::parse_from_str(inner, format.as_ref())
                    .map_err(|_| context.invalid(value, "NaiveDateTime"))
            }
            other => Err(context.invalid(other, "NaiveDateTime")),
        }
    }
}

fn excel_serial_to_datetime(
    value: &CellValue,
    context: &ConvertContext,
) -> Result<NaiveDateTime, ExcelError> {
    let serial = match value {
        CellValue::Int(inner) => inner
            .to_f64()
            .ok_or_else(|| context.invalid(value, "Excel date"))?,
        CellValue::Float(value) => *value,
        CellValue::Decimal(decimal) => decimal
            .to_f64()
            .ok_or_else(|| context.invalid(value, "Excel date"))?,
        other => return Err(context.invalid(other, "Excel date")),
    };
    easyexcel_model::DateSystem::from_1904_windowing(context.use_1904_windowing)
        .serial_to_datetime(serial)
        .ok_or_else(|| context.invalid(value, "Excel date"))
}

impl IntoExcelCell for NaiveDateTime {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::DateTime(*self))
    }
}

impl FromExcelCell for Vec<u8> {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        let value = value.unwrap_or(&CellValue::Empty);
        match value {
            CellValue::Image(bytes) => Ok(bytes.clone()),
            other => Err(context.invalid(other, "Vec<u8>")),
        }
    }
}

impl IntoExcelCell for Vec<u8> {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::Image(self.clone()))
    }
}

impl FromExcelCell for Box<[u8]> {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        Vec::<u8>::from_excel_cell(value, context).map(Vec::into_boxed_slice)
    }
}

impl IntoExcelCell for Box<[u8]> {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::Image(self.to_vec()))
    }
}

impl<const N: usize> FromExcelCell for [u8; N] {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        Vec::<u8>::from_excel_cell(value, context)?
            .try_into()
            .map_err(|_| context.invalid(value.unwrap_or(&CellValue::Empty), "[u8; N]"))
    }
}

impl<const N: usize> IntoExcelCell for [u8; N] {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::Image(self.to_vec()))
    }
}

impl FromExcelCell for std::path::PathBuf {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        String::from_excel_cell(value, context).map(Self::from)
    }
}

impl IntoExcelCell for std::path::PathBuf {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        easyexcel_io::io::file_utils::read_file(self)
            .map(CellValue::Image)
            .map_err(ExcelError::from)
    }
}

impl<T: FromExcelCell> FromExcelCell for Option<T> {
    fn from_excel_cell(
        value: Option<&CellValue>,
        context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        if value.is_none_or(CellValue::is_empty) {
            Ok(None)
        } else {
            T::from_excel_cell(value, context).map(Some)
        }
    }
}

impl<T: IntoExcelCell> IntoExcelCell for Option<T> {
    fn to_excel_cell(&self, context: &ConvertContext) -> Result<CellValue, ExcelError> {
        self.as_ref()
            .map_or(Ok(CellValue::Empty), |value| value.to_excel_cell(context))
    }
}

impl ExcelRow for DynamicRow {
    fn schema() -> &'static [crate::core::excel_column::ExcelColumn] {
        &[]
    }

    fn from_row(row: &RowData) -> Result<Self, ExcelError> {
        Ok(Self(
            (0..row.dynamic_width())
                .map(|index| (index, row.dynamic_cell(index)))
                .collect(),
        ))
    }

    fn to_row(&self) -> Result<Vec<CellValue>, ExcelError> {
        let entries = &self.0;
        let Some(last_index) = entries.last_key_value().map(|(index, _)| *index) else {
            return Ok(Vec::new());
        };
        let row_length = last_index
            .checked_add(1)
            .ok_or_else(|| ExcelError::Format("dynamic column index exceeds usize".to_owned()))?;
        let mut row = vec![CellValue::Empty; row_length];
        for (index, value) in entries {
            row[*index] = match value {
                crate::core::dynamic_value::DynamicValue::Null => CellValue::Empty,
                crate::core::dynamic_value::DynamicValue::String(value) => {
                    CellValue::String(value.clone())
                }
                crate::core::dynamic_value::DynamicValue::ActualData(value) => value.clone(),
                crate::core::dynamic_value::DynamicValue::ReadCellData(value) => {
                    value.data().clone()
                }
            };
        }
        Ok(row)
    }
}

#[cfg(test)]
mod tests_extra {
    use std::collections::BTreeMap;

    use super::*;
    use crate::core::dynamic_value::DynamicValue;
    use crate::core::read_cell_data::ReadCellData;

    fn context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Data".to_owned(),
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
    fn string_reference_into_excel_cell() {
        // 对应 Java：String 类型默认写转换器
        assert_eq!(
            "text".to_excel_cell(&context()).unwrap(),
            CellValue::String("text".to_owned())
        );
    }

    #[test]
    fn bool_from_all_supported_cells_and_rejects_others() {
        // 对应 Java：Boolean 类型默认读转换器（BooleanNumberConverter 等）
        for (cell, expected) in [
            (CellValue::Bool(true), true),
            (CellValue::Bool(false), false),
            (CellValue::Int(1), true),
            (CellValue::Int(0), false),
            (CellValue::Float(1.0), true),
            (CellValue::Float(0.0), false),
            (CellValue::Decimal(BigDecimal::from(1)), true),
            (CellValue::Decimal(BigDecimal::from(0)), false),
            (CellValue::String("true".to_owned()), true),
            (CellValue::String("1".to_owned()), true),
            (CellValue::String("FALSE".to_owned()), false),
            (CellValue::String("0".to_owned()), false),
        ] {
            assert_eq!(
                bool::from_excel_cell(Some(&cell), &context()).unwrap(),
                expected,
                "cell {cell:?}"
            );
        }
        for cell in [CellValue::Error("#DIV/0!".to_owned()), CellValue::Empty] {
            assert!(bool::from_excel_cell(Some(&cell), &context()).is_err());
        }
        assert!(bool::from_excel_cell(None, &context()).is_err());
    }

    #[test]
    fn big_int_from_all_supported_cells_and_rejects_others() {
        // 对应 Java：BigInteger 类型默认读转换器
        assert_eq!(
            BigInt::from_excel_cell(Some(&CellValue::Bool(true)), &context()).unwrap(),
            BigInt::from(1)
        );
        assert_eq!(
            BigInt::from_excel_cell(Some(&CellValue::Int(42)), &context()).unwrap(),
            BigInt::from(42)
        );
        assert_eq!(
            BigInt::from_excel_cell(Some(&CellValue::Float(1.5)), &context()).unwrap(),
            BigInt::from(1)
        );
        assert!(BigInt::from_excel_cell(Some(&CellValue::Float(f64::NAN)), &context()).is_err());
        assert_eq!(
            BigInt::from_excel_cell(
                Some(&CellValue::Decimal("5.7".parse().unwrap())),
                &context()
            )
            .unwrap(),
            BigInt::from(5)
        );
        assert_eq!(
            BigInt::from_excel_cell(Some(&CellValue::String("3.9".to_owned())), &context())
                .unwrap(),
            BigInt::from(3)
        );
        assert!(
            BigInt::from_excel_cell(Some(&CellValue::String("abc".to_owned())), &context())
                .is_err()
        );
        assert!(
            BigInt::from_excel_cell(Some(&CellValue::Error("#REF!".to_owned())), &context())
                .is_err()
        );
        assert!(BigInt::from_excel_cell(None, &context()).is_err());
    }

    #[test]
    fn integers_parse_bool_float_decimal_and_reject_fractional_or_invalid() {
        // 对应 Java：Integer 等默认读转换器仅接受整数值
        assert_eq!(
            i32::from_excel_cell(Some(&CellValue::Bool(true)), &context()).unwrap(),
            1
        );
        assert_eq!(
            i32::from_excel_cell(Some(&CellValue::Int(7)), &context()).unwrap(),
            7
        );
        assert_eq!(
            i32::from_excel_cell(Some(&CellValue::Float(3.0)), &context()).unwrap(),
            3
        );
        assert_eq!(
            i32::from_excel_cell(Some(&CellValue::Decimal("5".parse().unwrap())), &context())
                .unwrap(),
            5
        );
        assert_eq!(
            i32::from_excel_cell(Some(&CellValue::String("9".to_owned())), &context()).unwrap(),
            9
        );
        for cell in [
            CellValue::Float(3.5),
            CellValue::Decimal("5.5".parse().unwrap()),
            CellValue::Date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        ] {
            assert!(i32::from_excel_cell(Some(&cell), &context()).is_err());
        }
        assert!(u8::from_excel_cell(Some(&CellValue::Int(300)), &context()).is_err());
        assert!(i32::from_excel_cell(None, &context()).is_err());
    }

    #[test]
    // 1.0/1.5/1.25/2.5 均可被 f64 二进制精确表示，精确比较正是本测试的意图
    #[allow(clippy::float_cmp)]
    fn floats_parse_all_scalar_cells_and_reject_others() {
        // 对应 Java：Float / Double 默认读转换器
        for (cell, expected) in [
            (CellValue::Bool(true), 1.0),
            (CellValue::Int(2), 2.0),
            (CellValue::Float(1.5), 1.5),
            (CellValue::Decimal("1.25".parse().unwrap()), 1.25),
            (CellValue::String("2.5".to_owned()), 2.5),
        ] {
            assert_eq!(
                f64::from_excel_cell(Some(&cell), &context()).unwrap(),
                expected,
                "cell {cell:?}"
            );
        }
        assert!(
            f64::from_excel_cell(
                Some(&CellValue::Date(
                    NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()
                )),
                &context()
            )
            .is_err()
        );
        assert!(f64::from_excel_cell(None, &context()).is_err());
    }

    #[test]
    fn big_decimal_from_all_supported_cells_and_rejects_others() {
        // 对应 Java：BigDecimal 默认读转换器
        assert_eq!(
            BigDecimal::from_excel_cell(Some(&CellValue::Bool(false)), &context()).unwrap(),
            BigDecimal::from(0)
        );
        assert_eq!(
            BigDecimal::from_excel_cell(Some(&CellValue::Int(7)), &context()).unwrap(),
            BigDecimal::from(7)
        );
        assert_eq!(
            BigDecimal::from_excel_cell(Some(&CellValue::Float(1.5)), &context()).unwrap(),
            BigDecimal::from_str("1.5").unwrap()
        );
        assert!(
            BigDecimal::from_excel_cell(Some(&CellValue::Float(f64::NAN)), &context()).is_err()
        );
        assert_eq!(
            BigDecimal::from_excel_cell(Some(&CellValue::String("1.5".to_owned())), &context())
                .unwrap(),
            BigDecimal::from_str("1.5").unwrap()
        );
        assert!(
            BigDecimal::from_excel_cell(Some(&CellValue::String("abc".to_owned())), &context())
                .is_err()
        );
        assert!(
            BigDecimal::from_excel_cell(Some(&CellValue::Error("#N/A".to_owned())), &context())
                .is_err()
        );
        assert!(BigDecimal::from_excel_cell(None, &context()).is_err());
    }

    #[test]
    fn dates_from_date_cells_serials_and_reject_others() {
        // 对应 Java：LocalDate / LocalDateTime 默认读转换器
        let date = NaiveDate::from_ymd_opt(2026, 1, 2).unwrap();
        assert_eq!(
            NaiveDate::from_excel_cell(Some(&CellValue::Date(date)), &context()).unwrap(),
            date
        );
        assert!(
            NaiveDate::from_excel_cell(Some(&CellValue::Error("#VALUE!".to_owned())), &context())
                .is_err()
        );
        assert_eq!(
            NaiveDateTime::from_excel_cell(Some(&CellValue::Date(date)), &context()).unwrap(),
            date.and_hms_opt(0, 0, 0).unwrap()
        );
        assert!(
            NaiveDateTime::from_excel_cell(
                Some(&CellValue::Error("#VALUE!".to_owned())),
                &context()
            )
            .is_err()
        );
    }

    #[test]
    fn excel_serial_epochs_and_invalid_serials() {
        // 对应 Java：Excel 序列号 → 日期，1900/1904 窗口与 60/61 虚拟闰日
        let context = context();
        let from = |cell: &CellValue| NaiveDateTime::from_excel_cell(Some(cell), &context);
        let midnight_1900_03_01 = NaiveDate::from_ymd_opt(1900, 3, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        assert_eq!(from(&CellValue::Float(61.0)).unwrap(), midnight_1900_03_01);
        let noon_1900_01_01 = NaiveDate::from_ymd_opt(1900, 1, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        assert_eq!(
            from(&CellValue::Decimal("1.5".parse().unwrap())).unwrap(),
            noon_1900_01_01
        );
        for cell in [
            CellValue::Float(-1.0),
            CellValue::Float(f64::NAN),
            CellValue::Int(-1),
        ] {
            assert!(from(&cell).is_err(), "cell {cell:?}");
        }
        for cell in [
            CellValue::Bool(true),
            CellValue::String("2026-01-01".to_owned()),
        ] {
            assert!(from(&cell).is_err(), "cell {cell:?}");
        }
    }

    #[test]
    fn byte_arrays_and_path_buffers_roundtrip() {
        // 对应 Java：byte[] / Byte[] / InputStream / File 图片转换器
        let context = context();
        let image = CellValue::Image(vec![0x89, b'P', b'N', b'G']);
        assert_eq!(
            Vec::<u8>::from_excel_cell(Some(&image), &context).unwrap(),
            vec![0x89, b'P', b'N', b'G']
        );
        assert!(
            Vec::<u8>::from_excel_cell(Some(&CellValue::String("x".to_owned())), &context).is_err()
        );
        assert_eq!(
            Box::<[u8]>::from_excel_cell(Some(&image), &context).unwrap(),
            vec![0x89, b'P', b'N', b'G'].into_boxed_slice()
        );
        assert!(
            Box::<[u8]>::from_excel_cell(Some(&CellValue::Error("#N/A".to_owned())), &context)
                .is_err()
        );
        let short = CellValue::Image(vec![1, 2]);
        let image_3 = CellValue::Image(vec![0x89, b'P', b'N']);
        let array: [u8; 3] = <[u8; 3]>::from_excel_cell(Some(&image_3), &context).unwrap();
        assert_eq!(array, [0x89, b'P', b'N']);
        assert!(<[u8; 3]>::from_excel_cell(Some(&short), &context).is_err());
        assert_eq!(
            std::path::PathBuf::from_excel_cell(
                Some(&CellValue::String("/tmp/image.png".to_owned())),
                &context
            )
            .unwrap(),
            std::path::PathBuf::from("/tmp/image.png")
        );
    }

    #[test]
    fn dynamic_row_to_row_covers_all_dynamic_value_variants() {
        // 对应 Java：Map<Integer, Object> 动态行写出，READ_CELL_DATA 取 data()
        let read_cell = ReadCellData::new(
            0,
            3,
            CellValue::String("raw".to_owned()),
            CellValue::Bool(true),
            "display".to_owned(),
            None,
        );
        let row = DynamicRow::new(BTreeMap::from([
            (0, DynamicValue::Null),
            (1, DynamicValue::String("s".to_owned())),
            (2, DynamicValue::ActualData(CellValue::Int(5))),
            (3, DynamicValue::ReadCellData(read_cell)),
        ]));
        assert_eq!(
            row.to_row().unwrap(),
            vec![
                CellValue::Empty,
                CellValue::String("s".to_owned()),
                CellValue::Int(5),
                CellValue::Bool(true),
            ]
        );
        assert_eq!(
            DynamicRow::default().to_row().unwrap(),
            Vec::<CellValue>::new()
        );
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    fn context() -> ConvertContext {
        ConvertContext {
            sheet_name: "Data".to_owned(),
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
    fn excel_serial_to_datetime_rejects_non_numeric_cells() {
        // 对应 Java：Excel 序列号仅接受数值单元格，其余类型报无效转换错误
        let context = context();
        for cell in [
            CellValue::Bool(true),
            CellValue::String("1.5".to_owned()),
            CellValue::Date(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
        ] {
            assert!(
                excel_serial_to_datetime(&cell, &context).is_err(),
                "cell {cell:?} should be rejected"
            );
        }
    }
}
