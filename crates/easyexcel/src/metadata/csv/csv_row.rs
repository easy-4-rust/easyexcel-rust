//! Java `CsvRow` 兼容适配；稀疏行模型由 `easyexcel-csv` 维护。

use crate::CellValue;
use crate::core::excel_error::ExcelError;
use crate::util::work_book_util::CellCreator;

use super::CsvCell;

/// Java EasyExcel 值模型参数化后的 CSV 行。
pub type CsvRow = easyexcel_csv::CsvRow<CellValue>;

impl CellCreator for CsvRow {
    type Cell<'a>
        = &'a mut CsvCell
    where
        Self: 'a;

    fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>, ExcelError> {
        self.try_create_cell(column_index).map_err(ExcelError::from)
    }
}
