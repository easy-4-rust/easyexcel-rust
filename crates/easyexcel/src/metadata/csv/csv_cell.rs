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
        Self::Error(easyexcel_model::CellError::from_biff_code(value).as_str().to_owned())
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
        if let Self::Bool(value) = self { Some(*value) } else { None }
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
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type CsvCell = easyexcel_csv::CsvCell<CellValue>;

/// Java/POI `CellType` 的 CSV 兼容枚举。
pub use easyexcel_csv::CsvCellType;
