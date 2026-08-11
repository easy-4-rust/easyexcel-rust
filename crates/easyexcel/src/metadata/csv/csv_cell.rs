//! Java `CsvCell` 兼容适配；存储模型由 `easyexcel-csv` 维护。

use crate::{CellValue, NumericCellType};

impl easyexcel_csv::CsvCellValue for CellValue {
    type NumericCellType = NumericCellType;

    const EMPTY: Self = Self::Empty;

    fn from_csv_text(value: String) -> Self {
        Self::String(value)
    }

    fn from_csv_formula(value: String) -> Self {
        Self::Formula(value)
    }

    fn from_csv_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    fn from_csv_number(value: f64) -> Self {
        Self::Float(value)
    }

    fn from_csv_error(value: u8) -> Self {
        Self::Error(
            easyexcel_model::CellError::from_biff_code(value)
                .as_str()
                .to_owned(),
        )
    }

    fn csv_number(&self) -> Option<f64> {
        match self {
            Self::Int(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            Self::Decimal(value) => value.to_string().parse().ok(),
            _ => None,
        }
    }

    fn csv_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }

    fn csv_error(&self) -> Option<u8> {
        if let Self::Error(value) = self {
            easyexcel_model::CellError::parse(value).map(easyexcel_model::CellError::biff_code)
        } else {
            None
        }
    }

    fn csv_numeric_cell_type(&self) -> Option<Self::NumericCellType> {
        match self {
            Self::Date(_) | Self::DateTime(_) => Some(NumericCellType::Date),
            Self::Int(_) | Self::Float(_) | Self::Decimal(_) => Some(NumericCellType::Number),
            _ => None,
        }
    }

    fn csv_display_text(&self) -> String {
        self.as_text()
    }
}

/// Java `EasyExcel` 值模型参数化后的 CSV 单元格。
/// 对应 Java：com.alibaba.excel.metadata.csv.CsvCell。
pub type CsvCell = easyexcel_csv::CsvCell<CellValue>;

/// Java/POI `CellType` 的 CSV 兼容枚举。
pub use easyexcel_csv::CsvCellType;

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_csv::CsvCellValue;

    #[test]
    fn empty_constant_is_cell_value_empty() {
        // 对应 Java：CsvCell.EMPTY
        assert_eq!(<CellValue as CsvCellValue>::EMPTY, CellValue::Empty);
    }

    #[test]
    fn from_csv_text_creates_string() {
        // 对应 Java：CsvCell.fromCsvText
        let cell = <CellValue as CsvCellValue>::from_csv_text("hello".to_owned());
        assert_eq!(cell, CellValue::String("hello".to_owned()));
    }

    #[test]
    fn from_csv_formula_creates_formula() {
        // 对应 Java：CsvCell.fromCsvFormula
        let cell = <CellValue as CsvCellValue>::from_csv_formula("=SUM(A1:A10)".to_owned());
        assert_eq!(cell, CellValue::Formula("=SUM(A1:A10)".to_owned()));
    }

    #[test]
    fn from_csv_bool_creates_bool() {
        // 对应 Java：CsvCell.fromCsvBool
        let cell = <CellValue as CsvCellValue>::from_csv_bool(true);
        assert_eq!(cell, CellValue::Bool(true));
        let cell = <CellValue as CsvCellValue>::from_csv_bool(false);
        assert_eq!(cell, CellValue::Bool(false));
    }

    #[test]
    fn from_csv_number_creates_float() {
        // 对应 Java：CsvCell.fromCsvNumber
        let cell = <CellValue as CsvCellValue>::from_csv_number(3.14);
        assert_eq!(cell, CellValue::Float(3.14));
    }

    #[test]
    fn csv_number_returns_float_for_int() {
        // 对应 Java：csvNumber 整数值
        let cell = CellValue::Int(42);
        assert_eq!(<CellValue as CsvCellValue>::csv_number(&cell), Some(42.0));
    }

    #[test]
    fn csv_number_returns_float_for_float() {
        // 对应 Java：csvNumber 浮点值
        let cell = CellValue::Float(3.14);
        assert_eq!(<CellValue as CsvCellValue>::csv_number(&cell), Some(3.14));
    }

    #[test]
    fn csv_number_returns_none_for_string() {
        // 对应 Java：csvNumber 字符串返回 None
        let cell = CellValue::String("abc".to_owned());
        assert!(<CellValue as CsvCellValue>::csv_number(&cell).is_none());
    }

    #[test]
    fn csv_bool_returns_bool_value() {
        // 对应 Java：csvBool
        let cell = CellValue::Bool(true);
        assert_eq!(<CellValue as CsvCellValue>::csv_bool(&cell), Some(true));
    }

    #[test]
    fn csv_bool_returns_none_for_non_bool() {
        // 对应 Java：csvBool 非布尔返回 None
        let cell = CellValue::Int(1);
        assert!(<CellValue as CsvCellValue>::csv_bool(&cell).is_none());
    }

    #[test]
    fn csv_error_returns_none_for_non_error() {
        // 对应 Java：csvError 非错误返回 None
        let cell = CellValue::String("ok".to_owned());
        assert!(<CellValue as CsvCellValue>::csv_error(&cell).is_none());
    }

    #[test]
    fn csv_numeric_cell_type_for_number() {
        // 对应 Java：csvNumericCellType 数字类型
        let cell = CellValue::Int(42);
        assert_eq!(
            <CellValue as CsvCellValue>::csv_numeric_cell_type(&cell),
            Some(NumericCellType::Number)
        );
        let cell = CellValue::Float(3.14);
        assert_eq!(
            <CellValue as CsvCellValue>::csv_numeric_cell_type(&cell),
            Some(NumericCellType::Number)
        );
    }

    #[test]
    fn csv_numeric_cell_type_for_string_is_none() {
        // 对应 Java：csvNumericCellType 字符串返回 None
        let cell = CellValue::String("abc".to_owned());
        assert!(<CellValue as CsvCellValue>::csv_numeric_cell_type(&cell).is_none());
    }

    #[test]
    fn csv_display_text_returns_as_text() {
        // 对应 Java：csvDisplayText
        let cell = CellValue::String("hello".to_owned());
        assert_eq!(
            <CellValue as CsvCellValue>::csv_display_text(&cell),
            "hello"
        );
        let cell = CellValue::Int(42);
        let text = <CellValue as CsvCellValue>::csv_display_text(&cell);
        assert!(!text.is_empty());
    }

    #[test]
    fn csv_cell_type_alias_exists() {
        // 对应 Java：CsvCell 类型别名
        let _type_check: std::any::TypeId = std::any::TypeId::of::<CsvCell>();
    }
}
