//! Java `CsvSheet` 兼容适配；有界行缓存由 `easyexcel-csv` 维护。

use crate::CellValue;
use crate::core::excel_error::ExcelError;
use crate::util::work_book_util::RowCreator;

use super::CsvRow;

/// Java `EasyExcel` 值模型参数化后的 CSV 工作表。
/// 对应 Java：com.alibaba.excel.metadata.csv.CsvSheet。
pub type CsvSheet = easyexcel_csv::CsvSheet<CellValue>;

impl RowCreator for CsvSheet {
    type Row<'a>
        = &'a mut CsvRow
    where
        Self: 'a;

    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>, ExcelError> {
        self.try_create_row(row_index).map_err(ExcelError::from)
    }
}
