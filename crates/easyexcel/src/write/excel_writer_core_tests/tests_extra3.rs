#![allow(clippy::too_many_lines)]
#[cfg(test)]
use super::*;

use std::collections::BTreeMap;
use std::io::{Cursor, Write};

use crate::core::{DynamicRow, DynamicValue};
use tempfile::tempdir;

/// 空实现 handler（对应 Java 无副作用的 `WriteHandler`）。
struct NoopHandler3;

impl WriteHandler for NoopHandler3 {}

/// 失败阶段可配置的 handler（对应 Java 测试里的 `FailingHandler` 模式）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum FailStage3 {
    BeforeWorkbookCreate,
    AfterSheetCreate,
    HeadCell,
}

struct StageFailingHandler3(FailStage3);

impl WriteHandler for StageFailingHandler3 {
    fn before_workbook_create(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        if self.0 == FailStage3::BeforeWorkbookCreate {
            Err(ExcelError::Format("stage failure".to_owned()))
        } else {
            Ok(())
        }
    }

    fn after_sheet_create(&mut self, _context: &WriteSheetContext) -> Result<()> {
        if self.0 == FailStage3::AfterSheetCreate {
            Err(ExcelError::Format("stage failure".to_owned()))
        } else {
            Ok(())
        }
    }

    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        if self.0 == FailStage3::HeadCell && context.is_head {
            Err(ExcelError::Format("stage failure".to_owned()))
        } else {
            Ok(())
        }
    }
}

/// 跳过所有单元格写入（对应 Java 里通过 handler 丢弃单元格）。
struct SkipCellHandler3;

impl WriteHandler for SkipCellHandler3 {
    fn before_cell_create(&mut self, context: &mut WriteCellContext) -> Result<()> {
        context.skip = true;
        Ok(())
    }
}

/// 具有重复 `unique_value` 的 handler（对应 Java `NotRepeatExecutor` 去重）。
struct UniqueHandler3(&'static str);

impl crate::event::NotRepeatExecutor for UniqueHandler3 {
    fn unique_value(&self) -> &str {
        self.0
    }
}

impl WriteHandler for UniqueHandler3 {
    fn as_not_repeat_executor(&self) -> Option<&dyn crate::event::NotRepeatExecutor> {
        Some(self)
    }
}

/// `to_row` 返回错误的行（对应 Java `toRow` 抛异常）。
struct FailingRow3;
impl ExcelRow for FailingRow3 {
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
            message: "round-2 injected conversion failure".to_owned(),
        })
    }
}

/// 普通单列 typed 行（schema 非空 → 走非 dynamic 表头分支）。
struct SingleColRow3 {
    cells: Vec<CellValue>,
}

impl ExcelRow for SingleColRow3 {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("field", "Field", Some(0), 0, None)];
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

fn dyn_row(values: &[(usize, &str)]) -> DynamicRow {
    DynamicRow::new(
        values
            .iter()
            .map(|(index, value)| (*index, DynamicValue::String((*value).to_owned())))
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

const PACKAGE_RELS_XML: &[u8] = br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;

const CONTENT_TYPES_XML: &[u8] = br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#;

const SHEET_XML: &[u8] = br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;

/// 缺少 `xl/_rels/workbook.xml.rels` 的模板：`ensure_sheet` 必须报错。
///
/// 对应 Java：POI 在 `createSheet` 时依赖 workbook 关系表，缺失即失败。
fn xlsx_template_missing_workbook_rels() -> Vec<u8> {
    zip_template(&[
            ("[Content_Types].xml", CONTENT_TYPES_XML),
            ("_rels/.rels", PACKAGE_RELS_XML),
            (
                "xl/workbook.xml",
                br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="TemplateOnly" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
            ),
            ("xl/worksheets/sheet1.xml", SHEET_XML),
        ])
}

// ========================================================================
// 生产代码 `?` 错误边：write_with_sheet_handlers / write_with_table_handlers
// 首次注册 sheet handler 时 workbook 回调失败（对应 Java `ExcelWriter` 抛异常）。
// ========================================================================

include!("tests_extra3/cases_01.rs");
include!("tests_extra3/cases_02.rs");
