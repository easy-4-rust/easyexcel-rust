#![allow(clippy::too_many_lines)]
use std::io::Cursor;
use std::io::Read as _;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::{
    CellValue, ExcelColumn, ExcelRow, RowData, WriteCellContext, WriteRowContext,
    WriteSheetContext, WriteWorkbookContext,
};
use crate::event::NotRepeatExecutor;
use calamine::{DataType, Reader, Xlsx, open_workbook};
use tempfile::tempdir;

use super::*;

struct SimpleRow(&'static str);

impl ExcelRow for SimpleRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &RowData) -> Result<Self> {
        Ok(Self(""))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::String(self.0.to_owned())])
    }
}

struct TwoColumnRow(&'static str, &'static str);

impl ExcelRow for TwoColumnRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("first", "First", Some(0), 0, None),
            ExcelColumn::new("second", "Second", Some(1), 1, None),
        ];
        COLUMNS
    }

    fn from_row(_row: &RowData) -> Result<Self> {
        Ok(Self("", ""))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![
            CellValue::String(self.0.to_owned()),
            CellValue::String(self.1.to_owned()),
        ])
    }
}

struct WorkbookProbe(Arc<AtomicUsize>);

impl WriteHandler for WorkbookProbe {
    fn before_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct UniqueWorkbookProbe {
    calls: Arc<AtomicUsize>,
    order: i32,
    unique_value: &'static str,
}

impl NotRepeatExecutor for UniqueWorkbookProbe {
    fn unique_value(&self) -> &str {
        self.unique_value
    }
}

impl WriteHandler for UniqueWorkbookProbe {
    fn order(&self) -> i32 {
        self.order
    }

    fn as_not_repeat_executor(&self) -> Option<&dyn NotRepeatExecutor> {
        Some(self)
    }

    fn before_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

struct ExactLifecycleProbe(Arc<Mutex<Vec<&'static str>>>);

impl ExactLifecycleProbe {
    fn record(&self, event: &'static str) {
        self.0.lock().expect("event log mutex poisoned").push(event);
    }
}

impl WriteHandler for ExactLifecycleProbe {
    fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.record("before_workbook_create");
        Ok(())
    }

    fn after_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.record("after_workbook_create");
        Ok(())
    }

    fn after_workbook_dispose(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.record("after_workbook_dispose");
        Ok(())
    }

    fn before_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
        self.record("before_sheet_create");
        Ok(())
    }

    fn after_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
        self.record("after_sheet_create");
        Ok(())
    }

    fn after_sheet_dispose(&mut self, _context: &WriteSheetContext) -> Result<()> {
        self.record("after_sheet_dispose");
        Ok(())
    }

    fn before_row_create(&mut self, _context: &WriteRowContext) -> Result<()> {
        self.record("before_row_create");
        Ok(())
    }

    fn after_row_create(&mut self, _context: &WriteRowContext) -> Result<()> {
        self.record("after_row_create");
        Ok(())
    }

    fn after_row_dispose(&mut self, _context: &WriteRowContext) -> Result<()> {
        self.record("after_row_dispose");
        Ok(())
    }

    fn before_cell_create(&mut self, _context: &mut WriteCellContext) -> Result<()> {
        self.record("before_cell_create");
        Ok(())
    }

    fn after_cell_create(&mut self, _context: &WriteCellContext) -> Result<()> {
        self.record("after_cell_create");
        Ok(())
    }

    fn after_cell_data_converted(&mut self, _context: &WriteCellContext) -> Result<()> {
        self.record("after_cell_data_converted");
        Ok(())
    }

    fn after_cell_dispose(&mut self, _context: &WriteCellContext) -> Result<()> {
        self.record("after_cell_dispose");
        Ok(())
    }
}

fn zip_entry(path: &std::path::Path, name: &str) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut entry = archive
        .by_name(name)
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    let mut text = String::new();
    entry.read_to_string(&mut text)?;
    Ok(text)
}

include!("tests/cases_01.rs");
