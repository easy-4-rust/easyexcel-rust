//! Mirrors Java `com.alibaba.excel.metadata.csv.CsvRow`.

use crate::excel_error::ExcelError;
use crate::util::work_book_util::CellCreator;

use super::csv_cell::CsvCell;
use super::csv_cell_style::CsvCellStyle;

/// One logical CSV row with sparse cells.
#[derive(Debug, Clone, PartialEq)]
pub struct CsvRow {
    row_index: u32,
    cells: Vec<CsvCell>,
    cell_style: Option<CsvCellStyle>,
}

impl CsvRow {
    /// Creates an empty row at a zero-based index.
    #[must_use]
    pub const fn new(row_index: u32) -> Self {
        Self {
            row_index,
            cells: Vec::new(),
            cell_style: None,
        }
    }

    /// Returns the zero-based row index.
    #[must_use]
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// Returns all materialised cells in creation order.
    #[must_use]
    pub fn cells(&self) -> &[CsvCell] {
        &self.cells
    }

    /// Returns a cell by its logical column.
    #[must_use]
    pub fn cell(&self, column_index: u16) -> Option<&CsvCell> {
        self.cells
            .iter()
            .find(|cell| cell.column_index() == column_index)
    }

    /// Sets the row-level style.
    pub fn set_cell_style(&mut self, style: CsvCellStyle) {
        self.cell_style = Some(style);
    }

    /// Builds a dense CSV record with `width` columns.
    #[must_use]
    pub fn into_record(self, width: usize) -> Vec<String> {
        let mut record = vec![String::new(); width];
        for cell in self.cells {
            let index = usize::from(cell.column_index());
            if let Some(slot) = record.get_mut(index) {
                *slot = cell.display_text();
            }
        }
        record
    }
}

impl CellCreator for CsvRow {
    type Cell<'a>
        = &'a mut CsvCell
    where
        Self: 'a;

    fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>, ExcelError> {
        if self
            .cells
            .iter()
            .any(|cell| cell.column_index() == column_index)
        {
            return Err(ExcelError::Format(format!(
                "CSV cell already exists at row {}, column {column_index}",
                self.row_index
            )));
        }
        self.cells.push(CsvCell::new(column_index));
        Ok(self.cells.last_mut().expect("just pushed"))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::CellValue;

    #[test]
    fn new_row_index_cells_and_cell_lookup() {
        // 对应 Java：CsvRow 行号与稀疏单元格查询
        let mut row = CsvRow::new(4);
        assert_eq!(row.row_index(), 4);
        assert!(row.cells().is_empty());
        assert!(row.cell(0).is_none());

        row.create_cell(1).expect("cell ok");
        row.create_cell(3).expect("cell ok");
        assert_eq!(row.cells().len(), 2);
        assert_eq!(row.cell(1).expect("found").column_index(), 1);
        assert!(row.cell(2).is_none());
    }

    #[test]
    fn create_cell_rejects_duplicate_column() {
        // 对应 Java：同一列重复创建时报格式错误
        let mut row = CsvRow::new(0);
        row.create_cell(0).expect("first ok");
        let err = row.create_cell(0).expect_err("duplicate");
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn into_record_builds_dense_record() {
        // 对应 Java：稀疏行转为定宽记录
        let mut row = CsvRow::new(0);
        row.set_cell_style(CsvCellStyle::new(1));
        row.create_cell(0)
            .expect("cell")
            .set_value(CellValue::Int(1));
        row.create_cell(2)
            .expect("cell")
            .set_value(CellValue::String("x".to_owned()));
        let record = row.into_record(4);
        assert_eq!(record, vec!["1", "", "x", ""]);
    }
}
