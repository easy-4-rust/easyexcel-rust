use super::xlsx_source::{XlsxSource, is_compound_document};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, Read, Write};
use std::path::Path;
use std::sync::Arc;

use crate::core::{
    AnalysisContext, CellValue, CsvCharset, CustomReadObject, DynamicRow, DynamicValue,
    ErrorAction, ExcelColumn, ExcelError, ExcelRow, FormulaData, IntoExcelCell, ReadDefaultReturn,
    ReadListener, Result, RowData,
};
use base64::Engine;
use calamine::{
    CellErrorType, Data, DataRef, ExcelDateTime, ExcelDateTimeType, Range, Xlsx, open_workbook,
};
use flate2::read::GzDecoder;
use rust_xlsxwriter::{Format, Note, Workbook};
use tempfile::{TempDir, tempdir};
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

use super::*;

struct FaultyBufRead;

impl Read for FaultyBufRead {
    fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("injected probe failure"))
    }
}

impl BufRead for FaultyBufRead {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        Err(std::io::Error::other("injected probe failure"))
    }

    fn consume(&mut self, amount: usize) {
        let _ = amount;
    }
}

fn test_error(error: impl std::fmt::Display) -> ExcelError {
    ExcelError::Format(error.to_string())
}

#[derive(Debug, PartialEq, Eq)]
struct TestRow(String);

impl ExcelRow for TestRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(row: &RowData) -> Result<Self> {
        let value = row
            .cell(&Self::schema()[0])
            .map_or_else(String::new, CellValue::as_text);
        if value == "conversion-error" {
            return Err(ExcelError::Format("conversion failed".to_owned()));
        }
        Ok(Self(value))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        self.0
            .to_excel_cell(&crate::core::ConvertContext {
                sheet_name: String::new(),
                row_index: 0,
                column_index: Some(0),
                field: "value",
                format: None,
                date_time_format: None,
                number_format: None,
                use_1904_windowing: false,
            })
            .map(|value| vec![value])
    }
}

#[derive(Debug, PartialEq, Eq)]
struct NamedRow(String);

impl ExcelRow for NamedRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Canonical", None, 0, None)];
        COLUMNS
    }

    fn from_row(row: &RowData) -> Result<Self> {
        Ok(Self(
            row.cell(&Self::schema()[0])
                .map_or_else(String::new, CellValue::as_text),
        ))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::String(self.0.clone())])
    }
}

#[derive(Default)]
struct NamedProbe {
    heads: Vec<HashMap<String, usize>>,
    rows: Vec<NamedRow>,
}

impl ReadListener<NamedRow> for NamedProbe {
    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        _context: &AnalysisContext,
    ) -> Result<()> {
        self.heads.push(head.clone());
        Ok(())
    }

    fn invoke(&mut self, data: NamedRow, _context: &AnalysisContext) -> Result<()> {
        self.rows.push(data);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RawRow {
    cells: Vec<CellValue>,
    formulas: Vec<Option<String>>,
}

impl ExcelRow for RawRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("shared", "Shared", Some(0), 0, None),
            ExcelColumn::new("inline", "Inline", Some(1), 1, None),
            ExcelColumn::new("boolean", "Boolean", Some(2), 2, None),
            ExcelColumn::new("integer", "Integer", Some(3), 3, None),
            ExcelColumn::new("float", "Float", Some(4), 4, None),
            ExcelColumn::new("formula_number", "Formula number", Some(5), 5, None),
            ExcelColumn::new("formula_string", "Formula string", Some(6), 6, None),
            ExcelColumn::new("error", "Error", Some(7), 7, None),
            ExcelColumn::new("date", "Date", Some(8), 8, None),
        ];
        COLUMNS
    }

    fn from_row(row: &RowData) -> Result<Self> {
        Ok(Self {
            cells: Self::schema()
                .iter()
                .map(|column| row.cell(column).cloned().unwrap_or(CellValue::Empty))
                .collect(),
            formulas: Self::schema()
                .iter()
                .map(|column| {
                    row.formula(column)
                        .map(|formula| formula.formula_value().to_owned())
                })
                .collect(),
        })
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(self.cells.clone())
    }
}

#[derive(Default)]
struct RawProbe(Vec<RawRow>);

impl ReadListener<RawRow> for RawProbe {
    fn invoke(&mut self, data: RawRow, _context: &AnalysisContext) -> Result<()> {
        self.0.push(data);
        Ok(())
    }
}

#[derive(Default)]
struct DynamicProbe(Vec<DynamicRow>);

impl ReadListener<DynamicRow> for DynamicProbe {
    fn invoke(&mut self, data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
        self.0.push(data);
        Ok(())
    }
}

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
struct Probe {
    heads: Vec<HashMap<String, usize>>,
    rows: Vec<TestRow>,
    after: Vec<(String, usize, u32)>,
    continue_reading: bool,
    fail_head: bool,
    fail_invoke: bool,
    fail_invoke_at: Option<usize>,
    invoke_count: usize,
    fail_after: bool,
    error_action: Option<ErrorAction>,
    errors: usize,
    stop_after_callbacks: Option<usize>,
    callback_count: usize,
}

impl ReadListener<TestRow> for Probe {
    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        _context: &AnalysisContext,
    ) -> Result<()> {
        if self.fail_head {
            return Err(ExcelError::Format("head failed".to_owned()));
        }
        self.heads.push(head.clone());
        Ok(())
    }

    fn invoke(&mut self, data: TestRow, _context: &AnalysisContext) -> Result<()> {
        self.invoke_count += 1;
        if self.fail_invoke || self.fail_invoke_at == Some(self.invoke_count) {
            return Err(ExcelError::Format("invoke failed".to_owned()));
        }
        self.rows.push(data);
        Ok(())
    }

    fn on_exception(&mut self, _error: &ExcelError, _context: &AnalysisContext) -> ErrorAction {
        self.errors += 1;
        self.error_action.unwrap_or(ErrorAction::Stop)
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        if self.fail_after {
            return Err(ExcelError::Format("after failed".to_owned()));
        }
        self.after.push((
            context.sheet_name().to_owned(),
            context.sheet_no(),
            context.row_index(),
        ));
        Ok(())
    }

    fn has_next(&mut self, _context: &AnalysisContext) -> bool {
        self.callback_count += 1;
        self.stop_after_callbacks
            .map_or(self.continue_reading, |limit| self.callback_count < limit)
    }
}

#[derive(Default)]
struct ExtraProbe {
    events: Vec<&'static str>,
    extras: Vec<crate::core::CellExtra>,
    context_customs: Vec<Option<String>>,
    fail_extra: bool,
    error_action: Option<ErrorAction>,
    errors: usize,
    stop_after_extra: bool,
    extra_seen: bool,
}

impl ExtraProbe {
    fn record_custom(&mut self, context: &AnalysisContext) {
        self.context_customs
            .push(context.custom::<String>().cloned());
    }
}

impl ReadListener<TestRow> for ExtraProbe {
    fn on_exception(&mut self, _error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        self.record_custom(context);
        self.errors += 1;
        self.error_action.unwrap_or(ErrorAction::Stop)
    }

    fn invoke_head(
        &mut self,
        _head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        self.record_custom(context);
        self.events.push("head");
        Ok(())
    }

    fn invoke(&mut self, _data: TestRow, context: &AnalysisContext) -> Result<()> {
        self.record_custom(context);
        self.events.push("row");
        Ok(())
    }

    fn extra(&mut self, extra: &crate::core::CellExtra, context: &AnalysisContext) -> Result<()> {
        self.record_custom(context);
        self.events.push("extra");
        self.extras.push(extra.clone());
        self.extra_seen = true;
        if self.fail_extra {
            Err(ExcelError::Format("extra failed".to_owned()))
        } else {
            Ok(())
        }
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        self.record_custom(context);
        self.events.push("after");
        Ok(())
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        self.record_custom(context);
        !(self.stop_after_extra && self.extra_seen)
    }
}

struct ErrorProbe {
    action: ErrorAction,
    errors: usize,
}

impl ReadListener<TestRow> for ErrorProbe {
    fn on_exception(&mut self, _error: &ExcelError, _context: &AnalysisContext) -> ErrorAction {
        self.errors += 1;
        self.action
    }

    fn invoke(&mut self, _data: TestRow, _context: &AnalysisContext) -> Result<()> {
        panic!("a conversion failure cannot invoke a row")
    }
}

fn options() -> ReadOptions {
    ReadOptions {
        sheet: SheetSelector::First,
        head_row_number: 1,
        ignore_empty_row: true,
        auto_trim: true,
        use_1904_windowing: false,
        scientific_format: ScientificFormatMode::Plain,
        locale: ExcelLocale::default(),
        start_row: None,
        end_row: None,
        header_aliases: HashMap::new(),
        custom_object: None,
        read_default_return: ReadDefaultReturn::default(),
        extra_read: HashSet::new(),
        password: None,
        charset: CsvCharset::default(),
        converters: crate::core::ConverterRegistry::default(),
        read_cache: ReadCacheMode::default(),
        read_cache_selector: None,
    }
}

fn workbook_fixture() -> Result<(TempDir, std::path::PathBuf)> {
    let directory = tempdir()?;
    let path = directory.path().join("fixture.xlsx");
    let mut workbook = Workbook::new();
    let first = workbook.add_worksheet();
    first.set_name("First").map_err(test_error)?;
    first.write_string(0, 0, "Value").map_err(test_error)?;
    first.write_string(1, 0, "one").map_err(test_error)?;
    let second = workbook.add_worksheet();
    second.set_name("Second").map_err(test_error)?;
    second.write_string(0, 0, "Value").map_err(test_error)?;
    second.write_string(1, 0, "two").map_err(test_error)?;
    workbook.save(&path).map_err(test_error)?;
    Ok((directory, path))
}

fn extra_workbook_fixture() -> Result<(TempDir, std::path::PathBuf)> {
    let directory = tempdir()?;
    let path = directory.path().join("extras.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet.set_name("Meta").map_err(test_error)?;
    worksheet.write_string(0, 0, "Value").map_err(test_error)?;
    worksheet.write_string(1, 0, "row").map_err(test_error)?;
    worksheet
        .insert_note(1, 0, &Note::new("comment & text"))
        .map_err(test_error)?;
    worksheet
        .write_url(2, 0, "https://example.com")
        .map_err(test_error)?;
    worksheet
        .write_url(2, 1, "internal:Meta!A1")
        .map_err(test_error)?;
    worksheet
        .merge_range(3, 0, 3, 1, "Merged", &Format::new())
        .map_err(test_error)?;
    workbook.save(&path).map_err(test_error)?;
    Ok((directory, path))
}

fn rewrite_first_sheet(source: &Path, destination: &Path, replacement: &str) -> Result<()> {
    let mut archive = ZipArchive::new(fs::File::open(source)?).map_err(test_error)?;
    let mut writer = ZipWriter::new(fs::File::create(destination)?);
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(test_error)?;
        let name = entry.name().to_owned();
        let options = SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            writer.add_directory(name, options).map_err(test_error)?;
            continue;
        }
        writer.start_file(&name, options).map_err(test_error)?;
        if name == "xl/worksheets/sheet1.xml" {
            writer
                .write_all(replacement.as_bytes())
                .map_err(test_error)?;
        } else {
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            writer.write_all(&bytes)?;
        }
    }
    writer.finish().map_err(test_error)?;
    Ok(())
}

fn write_xlsx_package(path: &Path, entries: &[(&str, &str)]) -> Result<()> {
    let mut writer = ZipWriter::new(fs::File::create(path)?);
    for (name, contents) in entries {
        writer
            .start_file(*name, SimpleFileOptions::default())
            .map_err(test_error)?;
        writer.write_all(contents.as_bytes())?;
    }
    writer.finish().map_err(test_error)?;
    Ok(())
}

fn remove_first_sheet(source: &Path, destination: &Path) -> Result<()> {
    let mut archive = ZipArchive::new(fs::File::open(source)?).map_err(test_error)?;
    let mut writer = ZipWriter::new(fs::File::create(destination)?);
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(test_error)?;
        let name = entry.name().to_owned();
        if name == "xl/worksheets/sheet1.xml" {
            continue;
        }
        let options = SimpleFileOptions::default().compression_method(entry.compression());
        if entry.is_dir() {
            writer.add_directory(name, options).map_err(test_error)?;
            continue;
        }
        writer.start_file(&name, options).map_err(test_error)?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        writer.write_all(&bytes)?;
    }
    writer.finish().map_err(test_error)?;
    Ok(())
}

fn worksheet_xml(cells: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row r="1">{cells}</row></sheetData>
</worksheet>"#
    )
}

fn column_name(index: u32) -> String {
    let mut value = index + 1;
    let mut name = String::new();
    while value > 0 {
        let remainder = ((value - 1) % 26) as u8;
        name.insert(0, char::from(b'A' + remainder));
        value = (value - 1) / 26;
    }
    name
}

fn encode_csv_fixture(encoding: &'static encoding_rs::Encoding, value: &str) -> Vec<u8> {
    if encoding == encoding_rs::UTF_16BE {
        value.encode_utf16().flat_map(u16::to_be_bytes).collect()
    } else if encoding == encoding_rs::UTF_16LE {
        value.encode_utf16().flat_map(u16::to_le_bytes).collect()
    } else {
        let (encoded, actual, had_errors) = encoding.encode(value);
        assert_eq!(actual, encoding);
        assert!(!had_errors);
        encoded.into_owned()
    }
}

include!("tests_cases/cases_01.rs");
include!("tests_cases/cases_02.rs");
include!("tests_cases/cases_03.rs");
include!("tests_cases/cases_04.rs");
