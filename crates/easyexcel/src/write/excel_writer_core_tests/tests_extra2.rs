#![allow(clippy::too_many_lines)]
#[cfg(test)]
use super::*;

use std::collections::BTreeMap;
use std::io::{Cursor, Write};

use crate::core::{DynamicRow, DynamicValue};
use tempfile::tempdir;

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
    book.sheet_mut(sheet_name);
    book.to_cfb_bytes().expect("cfb bytes")
}

fn xlsx_template_bytes(sheet_name: &str) -> Vec<u8> {
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name(sheet_name).expect("sheet name");
    sheet.write_string(0, 0, "seed").expect("seed cell");
    workbook.save_to_buffer().expect("template buffer")
}

/// 手工构造 ZIP 模板包（entries: (路径, 内容)），默认 Stored 压缩。
fn zip_template(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, bytes) in entries {
        writer.start_file(*name, options).expect("start entry");
        writer.write_all(bytes).expect("write entry");
    }
    writer.finish().expect("finish").into_inner()
}

fn minimal_workbook_xml(sheet_name: &str) -> String {
    format!(
        r#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="{sheet_name}" sheetId="1" r:id="rId1"/></sheets></workbook>"#
    )
}

const MINIMAL_PACKAGE_RELS_XML: &[u8] = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;

const MINIMAL_RELS_XML: &[u8] = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#;

const MINIMAL_SHEET_XML: &[u8] = br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;

const MINIMAL_CONTENT_TYPES_XML: &[u8] = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#;

/// 失败阶段可配置的处理器（对应 Java 测试里的 `FailingHandler` 模式）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum FailStage {
    BeforeWorkbookCreate,
    AfterSheetCreate,
    HeadCell,
    DataCell,
}

struct StageFailingHandler(FailStage);

impl WriteHandler for StageFailingHandler {
    fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        if self.0 == FailStage::BeforeWorkbookCreate {
            Err(ExcelError::Format("stage failure".to_owned()))
        } else {
            Ok(())
        }
    }

    fn after_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
        if self.0 == FailStage::AfterSheetCreate {
            Err(ExcelError::Format("stage failure".to_owned()))
        } else {
            Ok(())
        }
    }

    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        let expected = if context.is_head {
            FailStage::HeadCell
        } else {
            FailStage::DataCell
        };
        if self.0 == expected {
            Err(ExcelError::Format("stage failure".to_owned()))
        } else {
            Ok(())
        }
    }
}

/// 跳过所有单元格写入（对应 Java 里通过 handler 丢弃单元格）。
struct SkipCellHandler;

impl WriteHandler for SkipCellHandler {
    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        context.skip = true;
        Ok(())
    }
}

/// 只请求单元格样式（对应 Java `requestedStyle`）。
struct StyleRequestingHandler;

impl WriteHandler for StyleRequestingHandler {
    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        context.cell().set_style(ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Indexed(21)),
            ..ExcelCellStyle::new()
        });
        Ok(())
    }
}

/// 请求非法 loop-merge（eachRow=1 且 columnExtend=1）。
struct LoopMergeBadHandler;

impl WriteHandler for LoopMergeBadHandler {
    fn style_loop_merge(&self) -> Option<(crate::core::LoopMergeProperty, usize)> {
        Some((crate::core::LoopMergeProperty::new(1, 1), 0))
    }
}

/// `to_row` 返回错误的行（对应 Java `ConvertAllFiled` 抛异常场景）。
struct FailingRow2;

impl ExcelRow for FailingRow2 {
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
            message: "test-only row conversion failure".to_owned(),
        })
    }
}

/// 普通两列 typed 行。
struct PlainRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for PlainRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("first", "First", Some(0), 0, None),
            ExcelColumn::new("second", "Second", Some(1), 0, None),
        ];
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

impl PlainRow {
    fn new(first: &str, second: &str) -> Self {
        Self {
            cells: vec![
                CellValue::String(first.to_owned()),
                CellValue::String(second.to_owned()),
            ],
        }
    }
}

/// 注解 `loop_merge` 非法（eachRow=1 / columnExtend=1）的行。
struct LoopMergeBadRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for LoopMergeBadRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)
            .with_loop_merge(crate::core::LoopMergeProperty::new(1, 1))];
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

/// 强制列号超出 u16 上限的行（对应 Java `index = 70000` 的极端注解）。
struct WideIndexRow {
    cells: Vec<CellValue>,
}

impl ExcelRow for WideIndexRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] =
            &[ExcelColumn::new("field", "Field", Some(70_000), 0, None)];
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

// ========================================================================
// 注解处理器加载 / 表写入的错误分支
// ========================================================================

include!("tests_extra2/cases_01.rs");
include!("tests_extra2/cases_02.rs");
