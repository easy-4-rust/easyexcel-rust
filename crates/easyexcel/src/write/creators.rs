//! Excel 写入器 Creator 实现族。
//!
//! 对应 Java：`com.alibaba.excel.util.WorkBookUtil` 的 XLSX / BIFF8 Creator 实现（内部类型）。

use crate::core::{ExcelError, Result};
use crate::util::work_book_util::{CellCreator, RowCreator, SheetCreator, WorkBookCreator};
use easyexcel_xlsx::xlsx::generation::{self, Workbook, Worksheet};

use crate::write::excel_writer_core::format_error;
use crate::write::xls_adapter::{Biff8Book, Biff8Cell, Biff8Sheet};

include!("creators/xlsx_work_book_creator.rs");

include!("creators/xlsx_sheet_creator.rs");

include!("creators/xlsx_row_creator.rs");

include!("creators/xlsx_row.rs");

include!("creators/xlsx_cell.rs");

include!("creators/biff8row_creator.rs");

include!("creators/biff8row.rs");

include!("creators/biff8cell_handle.rs");

impl SheetCreator for Biff8Book {
    type Sheet<'a>
        = &'a mut Biff8Sheet
    where
        Self: 'a;

    fn create_sheet(&mut self, sheet_name: &str) -> Result<Self::Sheet<'_>> {
        Biff8Book::create_sheet(self, sheet_name).map_err(ExcelError::from)
    }
}
