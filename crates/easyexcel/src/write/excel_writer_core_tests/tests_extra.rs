#![allow(clippy::too_many_lines)]
#[cfg(test)]
use super::*;

use std::collections::BTreeMap;

use crate::core::{DynamicRow, DynamicValue};
use bigdecimal::BigDecimal;
use calamine::{Data, Reader, Xls, Xlsx};
use chrono::NaiveDate;
use std::str::FromStr;
use tempfile::tempdir;

const CFB_MAGIC: &[u8] = &[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

fn open_xls(path: &std::path::Path) -> Result<Xls<std::fs::File>> {
    Xls::new(std::fs::File::open(path)?).map_err(format_error)
}

fn open_xlsx(path: &std::path::Path) -> Result<Xlsx<std::fs::File>> {
    Xlsx::new(std::fs::File::open(path)?).map_err(format_error)
}

fn dyn_row(values: &[(usize, &str)]) -> DynamicRow {
    DynamicRow::new(
        values
            .iter()
            .map(|(index, value)| (*index, DynamicValue::String((*value).to_owned())))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn dyn_row_values(values: &[(usize, CellValue)]) -> DynamicRow {
    DynamicRow::new(
        values
            .iter()
            .map(|(index, value)| (*index, DynamicValue::ActualData(value.clone())))
            .collect::<BTreeMap<_, _>>(),
    )
}

fn xls_template_bytes(sheet_name: &str) -> Vec<u8> {
    let mut book = Biff8Book::default();
    let sheet = book.sheet_mut(sheet_name);
    sheet
        .set(
            0,
            0,
            Biff8Cell::general(Biff8Value::Text("seed".to_owned())),
        )
        .expect("seed cell");
    book.to_cfb_bytes().expect("cfb bytes")
}

fn xlsx_template_bytes(sheet_name: &str) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name(sheet_name).expect("sheet name");
    sheet.write_string(0, 0, "seed").expect("seed cell");
    workbook.save_to_buffer().expect("template buffer")
}

/// Minimal typed row with a two-column schema and annotation metadata.
struct TwoColRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for TwoColRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("field", "Field", Some(0), 0, None)
                .with_column_width(18)
                .with_content_style(ExcelCellStyle {
                    fill_pattern: Some(ExcelFillPattern::Solid),
                    fill_foreground_color: Some(ExcelColor::Indexed(14)),
                    ..ExcelCellStyle::new()
                }),
            ExcelColumn::new("type", "Type", Some(1), 0, None),
        ];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
            .head_row_height(31)
            .content_row_height(24);
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self { cells: Vec::new() })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(self.cells.clone())
    }
}

impl TwoColRow {
    fn new(field: &str, r#type: &str) -> Self {
        Self {
            cells: vec![
                CellValue::String(field.to_owned()),
                CellValue::String(r#type.to_owned()),
            ],
        }
    }
}

/// Handler that requests a concrete row height through the logical handle.
struct HeightRequestingHandler;

impl WriteHandler for HeightRequestingHandler {
    fn after_row_create(&mut self, context: &WriteRowContext) -> Result<()> {
        context.row().set_height(27);
        Ok(())
    }
}

/// Handler that flags cells for fill-style ignoring and requests a style.
struct StyleRequestingHandler;

impl WriteHandler for StyleRequestingHandler {
    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        context.ignore_fill_style = true;
        context.cell().set_style(ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Indexed(20)),
            ..ExcelCellStyle::new()
        });
        Ok(())
    }
}

/// Handler returning a negative (invalid) once-absolute merge property.
struct NegativeMergeHandler;

impl WriteHandler for NegativeMergeHandler {
    fn style_once_absolute_merge(
        &self,
    ) -> Option<crate::metadata::property::OnceAbsoluteMergeProperty> {
        Some(crate::core::OnceAbsoluteMergeProperty::new(-1, -1, 0, 1))
    }
}

/// Handler that only requests a style through the logical cell handle.
struct StyleOnlyHandler;

impl WriteHandler for StyleOnlyHandler {
    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        context.cell().set_style(ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Indexed(30)),
            ..ExcelCellStyle::new()
        });
        Ok(())
    }
}

/// Handler with a repeatable unique value, used for deduplication tests.
struct UniqueHandler(&'static str);

impl crate::event::NotRepeatExecutor for UniqueHandler {
    fn unique_value(&self) -> &str {
        self.0
    }
}

impl WriteHandler for UniqueHandler {
    fn as_not_repeat_executor(&self) -> Option<&dyn crate::event::NotRepeatExecutor> {
        Some(self)
    }
}

/// Handler requesting a loop-merge strategy through the query API.
struct LoopMergeHandler;

impl WriteHandler for LoopMergeHandler {
    fn style_loop_merge(&self) -> Option<(crate::core::LoopMergeProperty, usize)> {
        Some((crate::core::LoopMergeProperty::new(2, 1), 0))
    }
}

/// Row whose `to_row` fails with a typed data-conversion error.
struct FailingRow;

impl ExcelRow for FailingRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self)
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Err(ExcelError::Data {
            sheet: String::new(),
            row: 0,
            column: Some(7),
            field: "field",
            value: "bad".to_owned(),
            message: "injected conversion failure".to_owned(),
        })
    }
}

/// Row with a field-level `@ContentLoopMerge` annotation.
struct LoopMergeRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for LoopMergeRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)
            .with_loop_merge(crate::core::LoopMergeProperty::new(2, 1))];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new();
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self { cells: Vec::new() })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(self.cells.clone())
    }
}

impl LoopMergeRow {
    fn new(cells: Vec<CellValue>) -> Self {
        Self { cells }
    }
}

/// Row with a type-level `@OnceAbsoluteMerge` annotation.
struct AbsoluteMergeRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for AbsoluteMergeRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("left", "Left", Some(0), 0, None),
            ExcelColumn::new("right", "Right", Some(1), 0, None),
        ];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
            .once_absolute_merge(crate::core::OnceAbsoluteMergeProperty::new(10, 10, 0, 1));
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self { cells: Vec::new() })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(self.cells.clone())
    }
}

impl AbsoluteMergeRow {
    fn new(cells: Vec<CellValue>) -> Self {
        Self { cells }
    }
}

/// Row with a negative (invalid) absolute merge annotation.
struct NegativeMergeRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for NegativeMergeRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
            .once_absolute_merge(crate::core::OnceAbsoluteMergeProperty::new(-1, -1, 0, 1));
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self { cells: Vec::new() })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(self.cells.clone())
    }
}

impl NegativeMergeRow {
    fn new(cells: Vec<CellValue>) -> Self {
        Self { cells }
    }
}

/// Row with annotation head style/font metadata exercising style merges.
struct FontStyleRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for FontStyleRow {
    fn schema() -> &'static [ExcelColumn] {
        const HEAD_STYLE: ExcelCellStyle = ExcelCellStyle {
            font: Some(ExcelFontStyle {
                color: Some(ExcelColor::Rgb(0x11_2233)),
                font_height_in_points: Some(12.0),
                ..ExcelFontStyle::new()
            }),
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Rgb(0x01_0203)),
            fill_background_color: Some(ExcelColor::Rgb(0x04_0506)),
            ..ExcelCellStyle::new()
        };
        const HEAD_FONT: ExcelFontStyle = ExcelFontStyle {
            color: Some(ExcelColor::Rgb(0x77_8899)),
            font_height_in_points: Some(11.0),
            ..ExcelFontStyle::new()
        };
        const CONTENT_STYLE: ExcelCellStyle = ExcelCellStyle {
            font: Some(ExcelFontStyle {
                color: Some(ExcelColor::Rgb(0xDD_EEFF)),
                font_height_in_points: Some(10.0),
                ..ExcelFontStyle::new()
            }),
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Rgb(0x0A_0B0C)),
            ..ExcelCellStyle::new()
        };
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("field", "Field", Some(0), 0, None)
                .with_head_style(HEAD_STYLE)
                .with_head_font_style(HEAD_FONT)
                .with_content_style(CONTENT_STYLE),
            ExcelColumn::new("other", "Other", Some(1), 0, None),
        ];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const HEAD_FONT: ExcelFontStyle = ExcelFontStyle {
            color: Some(ExcelColor::Rgb(0x77_8899)),
            font_height_in_points: Some(11.0),
            ..ExcelFontStyle::new()
        };
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new().head_font_style(HEAD_FONT);
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self { cells: Vec::new() })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(self.cells.clone())
    }
}

impl FontStyleRow {
    fn new(cells: Vec<CellValue>) -> Self {
        Self { cells }
    }
}

include!("tests_extra/cases_01.rs");
include!("tests_extra/cases_02.rs");
include!("tests_extra/cases_03.rs");
include!("tests_extra/cases_04.rs");
