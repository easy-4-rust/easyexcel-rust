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
