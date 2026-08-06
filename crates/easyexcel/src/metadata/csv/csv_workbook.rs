//! Java `CsvWorkbook` 兼容适配；工作簿模型由 `easyexcel-csv` 维护。

use crate::CellValue;
use crate::core::excel_error::ExcelError;
use crate::util::work_book_util::SheetCreator;

use super::CsvSheet;

/// Java `EasyExcel` 值模型参数化后的 CSV 工作簿。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type CsvWorkbook = easyexcel_csv::CsvWorkbook<CellValue>;

impl SheetCreator for CsvWorkbook {
    type Sheet<'a>
        = &'a mut CsvSheet
    where
        Self: 'a;

    fn create_sheet(&mut self, sheet_name: &str) -> Result<Self::Sheet<'_>, ExcelError> {
        self.try_create_sheet(sheet_name).map_err(ExcelError::from)
    }
}
