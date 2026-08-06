#![allow(clippy::too_many_lines)]
use super::*;
use crate::core::{
    CellValue, DynamicRow, ExcelColumn, ExcelWriteMetadata, RowData, WriteDirection,
    WriteSheetContext,
};
use tempfile::tempdir;

struct ContextRow;

#[derive(Default)]
struct ContextFillExecutor;

impl WriteFillExecutor for ContextFillExecutor {
    fn fill(
        &mut self,
        _data: &dyn Any,
        _fill_config: WriteFillConfig,
        _sheet: WriteFillSheet,
    ) -> Result<()> {
        Ok(())
    }

    fn finish(&mut self, _on_exception: bool) -> Result<()> {
        Ok(())
    }
}

impl ExcelRow for ContextRow {
    fn schema() -> &'static [ExcelColumn] {
        static SCHEMA: [ExcelColumn; 3] = [
            ExcelColumn::new("a", "A", Some(0), 0, None),
            ExcelColumn::new("b", "B", Some(1), 0, None),
            ExcelColumn::new("c", "C", Some(2), 0, None),
        ];
        &SCHEMA
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        static METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new().head_row_height(28);
        &METADATA
    }

    fn from_row(_row: &RowData) -> Result<Self> {
        Ok(Self)
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![
            CellValue::String("a".to_owned()),
            CellValue::String("b".to_owned()),
            CellValue::String("c".to_owned()),
        ])
    }
}

include!("tests/cases_01.rs");
