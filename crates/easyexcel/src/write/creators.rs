//! Excel 写入器 Creator 实现族。
//!
//! 对应 Java：`com.alibaba.excel.util.WorkBookUtil` 的 XLSX / BIFF8 Creator 实现（内部类型）。

use crate::core::{ExcelError, Result};
use crate::util::work_book_util::{CellCreator, RowCreator, SheetCreator, WorkBookCreator};
use easyexcel_xlsx::xlsx::generation::{self, Workbook, Worksheet};

use crate::write::xls_adapter::{Biff8Book, Biff8Cell, Biff8Sheet};
use crate::write::excel_writer_core::format_error;

pub(crate) struct XlsxWorkBookCreator;

impl WorkBookCreator for XlsxWorkBookCreator {
    type WorkBook = Workbook;

    fn create_work_book(self) -> Result<Self::WorkBook> {
        Ok(easyexcel_xlsx::xlsx::generation::new_workbook())
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
        generation::create_worksheet(self.workbook, sheet_name, self.constant_memory)
            .map_err(format_error)
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
        generation::validate_row_index(row_index).map_err(ExcelError::from)?;
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
        generation::validate_column_index(column_index).map_err(ExcelError::from)?;
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
        Biff8Book::create_sheet(self, sheet_name).map_err(ExcelError::from)
    }
}

impl RowCreator for Biff8RowCreator<'_> {
    type Row<'a>
        = Biff8Row<'a>
    where
        Self: 'a;

    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>> {
        Biff8Sheet::validate_row_index(row_index).map_err(ExcelError::from)?;
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
        Biff8Sheet::column_index(usize::from(column_index)).map_err(ExcelError::from)?;
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
