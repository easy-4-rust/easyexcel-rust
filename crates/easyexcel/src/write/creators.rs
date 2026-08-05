//! Excel 写入器 Creator 实现族。
//!
//! 对应 Java：`com.alibaba.excel.util.WorkBookUtil` 的 XLSX / BIFF8 Creator 实现（内部类型）。

use crate::core::{ExcelError, Result};
use crate::util::work_book_util::{CellCreator, RowCreator, SheetCreator, WorkBookCreator};
use rust_xlsxwriter::{Workbook, Worksheet};

use crate::write::biff8::{Biff8Book, Biff8Cell, Biff8Sheet};
use crate::write::excel_writer_core::format_error;

pub(crate) struct XlsxWorkBookCreator;

impl WorkBookCreator for XlsxWorkBookCreator {
    type WorkBook = Workbook;

    fn create_work_book(self) -> Result<Self::WorkBook> {
        Ok(Workbook::new())
    }
}

pub(crate) struct XlsxSheetCreator<'a> {
    pub(crate) workbook: &'a mut Workbook,
    pub(crate) constant_memory: bool,
}

impl SheetCreator for XlsxSheetCreator<'_> {
    type Sheet<'a>
        = &'a mut Worksheet
    where
        Self: 'a;

    fn create_sheet(&mut self, sheet_name: &str) -> Result<Self::Sheet<'_>> {
        let worksheet = if self.constant_memory {
            self.workbook.add_worksheet_with_constant_memory()
        } else {
            self.workbook.add_worksheet()
        };
        worksheet.set_name(sheet_name).map_err(format_error)?;
        Ok(worksheet)
    }
}

pub(crate) struct XlsxRowCreator<'a> {
    pub(crate) worksheet: &'a mut Worksheet,
}

pub(crate) struct XlsxRow<'a> {
    pub(crate) worksheet: &'a mut Worksheet,
    pub(crate) row_index: u32,
}

pub(crate) struct XlsxCell<'a> {
    pub(crate) worksheet: &'a mut Worksheet,
    pub(crate) row_index: u32,
    pub(crate) column_index: u16,
}

impl RowCreator for XlsxRowCreator<'_> {
    type Row<'a>
        = XlsxRow<'a>
    where
        Self: 'a;

    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>> {
        if row_index >= 1_048_576 {
            return Err(ExcelError::Format(format!(
                "XLSX row index {row_index} exceeds 1048575"
            )));
        }
        Ok(XlsxRow {
            worksheet: self.worksheet,
            row_index,
        })
    }
}

impl CellCreator for XlsxRow<'_> {
    type Cell<'a>
        = XlsxCell<'a>
    where
        Self: 'a;

    fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>> {
        if column_index >= 16_384 {
            return Err(ExcelError::Format(format!(
                "XLSX column index {column_index} exceeds 16383"
            )));
        }
        Ok(XlsxCell {
            worksheet: self.worksheet,
            row_index: self.row_index,
            column_index,
        })
    }
}

pub(crate) struct Biff8RowCreator<'a> {
    pub(crate) sheet: &'a mut Biff8Sheet,
}

pub(crate) struct Biff8Row<'a> {
    pub(crate) sheet: &'a mut Biff8Sheet,
    pub(crate) row_index: u32,
}

pub(crate) struct Biff8CellHandle<'a> {
    pub(crate) sheet: &'a mut Biff8Sheet,
    pub(crate) row_index: u32,
    pub(crate) column_index: u16,
}

impl SheetCreator for Biff8Book {
    type Sheet<'a>
        = &'a mut Biff8Sheet
    where
        Self: 'a;

    fn create_sheet(&mut self, sheet_name: &str) -> Result<Self::Sheet<'_>> {
        if self.sheets.iter().any(|sheet| sheet.name == sheet_name) {
            return Err(ExcelError::Format(format!(
                "worksheet name is already in use: {sheet_name}"
            )));
        }
        self.sheets.push(Biff8Sheet::new(sheet_name));
        Ok(self.sheets.last_mut().expect("just pushed"))
    }
}

impl RowCreator for Biff8RowCreator<'_> {
    type Row<'a>
        = Biff8Row<'a>
    where
        Self: 'a;

    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>> {
        if row_index >= 65_536 {
            return Err(ExcelError::Format(
                "BIFF8 supports at most 65536 rows".to_owned(),
            ));
        }
        Ok(Biff8Row {
            sheet: self.sheet,
            row_index,
        })
    }
}

impl CellCreator for Biff8Row<'_> {
    type Cell<'a>
        = Biff8CellHandle<'a>
    where
        Self: 'a;

    fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>> {
        if column_index >= 256 {
            return Err(ExcelError::Format(
                "BIFF8 supports at most 256 columns".to_owned(),
            ));
        }
        Ok(Biff8CellHandle {
            sheet: self.sheet,
            row_index: self.row_index,
            column_index,
        })
    }
}

impl Biff8CellHandle<'_> {
    pub(crate) fn set(self, cell: Biff8Cell) -> Result<()> {
        self.sheet
            .set(self.row_index, usize::from(self.column_index), cell)?;
        Ok(())
    }
}
