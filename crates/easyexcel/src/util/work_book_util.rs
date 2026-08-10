//! 对应 Java： com.alibaba.excel.util.WorkBookUtil.
//!
//! Java wraps Apache POI `Workbook` / `Sheet` / `Row` / `Cell`
//! construction behind a small utility boundary. Rust keeps that boundary
//! backend-neutral: writer crates implement the creator traits for XLSX,
//! BIFF8 or CSV objects and these functions perform the same delegation.

use crate::core::excel_error::ExcelError;
use crate::write::write_cell_data::WriteCellData;

include!("work_book_util/work_book_creator.rs");

include!("work_book_util/sheet_creator.rs");

include!("work_book_util/row_creator.rs");

include!("work_book_util/cell_creator.rs");

/// 对应 Java：com.alibaba.excel.util.WorkBookUtil。 Mirrors `com.alibaba.excel.util.WorkBookUtil#createWorkBook`.
///
/// # Errors
///
/// Propagates workbook construction errors from the selected backend.
pub fn create_work_book<C: WorkBookCreator>(creator: C) -> Result<C::WorkBook, ExcelError> {
    creator.create_work_book()
}

/// 对应 Java：com.alibaba.excel.util.WorkBookUtil。 Mirrors `com.alibaba.excel.util.WorkBookUtil#createSheet`.
///
/// # Errors
///
/// Propagates sheet creation errors from the selected backend.
pub fn create_sheet<'a, C: SheetCreator>(
    workbook: &'a mut C,
    sheet_name: &str,
) -> Result<C::Sheet<'a>, ExcelError> {
    workbook.create_sheet(sheet_name)
}

/// 对应 Java：com.alibaba.excel.util.WorkBookUtil。 Mirrors `com.alibaba.excel.util.WorkBookUtil#createRow`.
///
/// # Errors
///
/// Propagates row creation errors from the selected backend.
pub fn create_row<C: RowCreator>(sheet: &mut C, row_index: u32) -> Result<C::Row<'_>, ExcelError> {
    sheet.create_row(row_index)
}

/// 对应 Java：com.alibaba.excel.util.WorkBookUtil。 Mirrors `com.alibaba.excel.util.WorkBookUtil#createCell`.
///
/// # Errors
///
/// Propagates cell creation errors from the selected backend.
pub fn create_cell<C: CellCreator>(
    row: &mut C,
    column_index: u16,
) -> Result<C::Cell<'_>, ExcelError> {
    row.create_cell(column_index)
}

/// 对应 Java：com.alibaba.excel.util.WorkBookUtil。 Mirrors `com.alibaba.excel.util.WorkBookUtil#fillDataFormat`.
///
/// Java creates the missing `WriteCellStyle` and `DataFormatData` containers,
/// then sets the requested format only when no format was already assigned.
pub fn fill_data_format(cell_data: &mut WriteCellData, format: Option<&str>, default_format: &str) {
    cell_data.get_or_create_style();
    let data_format = cell_data.get_or_create_data_format();
    if data_format.format.is_none() {
        data_format.format = Some(format.unwrap_or(default_format).to_owned());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CellValue, WriteCellStyle};

    #[derive(Default)]
    struct TestWorkBook {
        sheets: Vec<TestSheet>,
    }

    struct TestWorkBookFactory;

    impl WorkBookCreator for TestWorkBookFactory {
        type WorkBook = TestWorkBook;

        fn create_work_book(self) -> Result<Self::WorkBook, ExcelError> {
            Ok(TestWorkBook::default())
        }
    }

    #[derive(Default)]
    struct TestSheet {
        name: String,
        rows: Vec<TestRow>,
    }

    #[derive(Default)]
    struct TestRow {
        index: u32,
        cells: Vec<TestCell>,
    }

    #[derive(Debug, PartialEq, Eq)]
    struct TestCell {
        column: u16,
    }

    impl SheetCreator for TestWorkBook {
        type Sheet<'a> = &'a mut TestSheet;

        fn create_sheet(&mut self, sheet_name: &str) -> Result<Self::Sheet<'_>, ExcelError> {
            self.sheets.push(TestSheet {
                name: sheet_name.to_owned(),
                rows: Vec::new(),
            });
            Ok(self.sheets.last_mut().expect("just pushed"))
        }
    }

    impl RowCreator for TestSheet {
        type Row<'a> = &'a mut TestRow;

        fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>, ExcelError> {
            self.rows.push(TestRow {
                index: row_index,
                cells: Vec::new(),
            });
            Ok(self.rows.last_mut().expect("just pushed"))
        }
    }

    impl CellCreator for TestRow {
        type Cell<'a> = &'a mut TestCell;

        fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>, ExcelError> {
            self.cells.push(TestCell {
                column: column_index,
            });
            Ok(self.cells.last_mut().expect("just pushed"))
        }
    }

    #[test]
    fn creator_chain_delegates_to_real_backend_objects() {
        let mut workbook = create_work_book(TestWorkBookFactory).expect("workbook");
        let sheet = create_sheet(&mut workbook, "用户").expect("sheet");
        assert_eq!(sheet.name, "用户");
        let row = create_row(sheet, 7).expect("row");
        assert_eq!(row.index, 7);
        let cell = create_cell(row, 3).expect("cell");
        assert_eq!(*cell, TestCell { column: 3 });
        assert_eq!(workbook.sheets[0].rows[0].cells.len(), 1);
    }

    #[test]
    fn fill_data_format_creates_nested_state_and_preserves_existing_format() {
        let mut cell = WriteCellData::new(CellValue::Int(1));
        fill_data_format(&mut cell, None, "yyyy-mm-dd");
        assert_eq!(
            cell.data_format_data().and_then(|value| value.format()),
            Some("yyyy-mm-dd")
        );
        let default_style = WriteCellStyle::default();
        assert_eq!(cell.write_cell_style(), Some(&default_style));

        fill_data_format(&mut cell, Some("0.00"), "General");
        assert_eq!(
            cell.data_format_data().and_then(|value| value.format()),
            Some("yyyy-mm-dd"),
            "Java does not overwrite an existing format"
        );
    }
}
