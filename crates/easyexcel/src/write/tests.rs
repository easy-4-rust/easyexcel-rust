use std::cell::Cell;
use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Cursor, Read as _, Seek, SeekFrom, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
#[allow(unused_imports)]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

#[allow(unused_imports)]
use crate::core::{
    BigDecimal, ClientAnchorData, CoordinateData, DynamicRow, DynamicValue, HeadKind, ImageData,
    ImageType, IntoExcelCell, OnceAbsoluteMergeProperty, WriteCellData,
};
#[allow(unused_imports)]
use crate::metadata::{CellRange, RowHeightProperty};
#[allow(unused_imports)]
use calamine::{Data, DataType, Dimensions, Reader, Xls, Xlsx, open_workbook};
use chrono::NaiveDate;
#[allow(unused_imports)]
use tempfile::tempdir;
use zip::ZipArchive;

// 测试模块需要访问 lib 内部全部符号，逐项导入会随实现持续漂移
#[allow(clippy::wildcard_imports)]
use super::*;

#[allow(unused_imports)]
use crate::write::create_work_book;
#[allow(unused_imports)]
use crate::write::creators::{XlsxSheetCreator, XlsxWorkBookCreator};
#[allow(unused_imports)]
use crate::write::handler_execution_scope::load_annotation_handlers;
#[allow(unused_imports)]
use crate::write::shared_write_handler::SharedWriteHandler;

#[allow(dead_code)]
fn test_error(error: impl std::fmt::Display) -> ExcelError {
    ExcelError::Format(error.to_string())
}

#[allow(dead_code)]
struct FaultyWrite {
    fail_write_at: Option<usize>,
    fail_flush: bool,
    writes: usize,
}

impl FaultyWrite {
    #[allow(dead_code)]
    const fn writing(fail_at: usize) -> Self {
        Self {
            fail_write_at: Some(fail_at),
            fail_flush: false,
            writes: 0,
        }
    }

    #[allow(dead_code)]
    const fn flushing() -> Self {
        Self {
            fail_write_at: None,
            fail_flush: true,
            writes: 0,
        }
    }
}

impl Write for FaultyWrite {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let call = self.writes;
        self.writes += 1;
        if self.fail_write_at == Some(call) {
            return Err(io::Error::other("injected CSV write failure"));
        }
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            return Err(io::Error::other("injected CSV flush failure"));
        }
        Ok(())
    }
}

#[allow(dead_code)]
struct PanicWrite;

impl Write for PanicWrite {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        panic!("poison output lock");
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
#[allow(dead_code)]
struct StreamProbe {
    bytes: Vec<u8>,
    fail_write: bool,
    fail_flush: bool,
}

impl Write for StreamProbe {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        if self.fail_write {
            Err(io::Error::other("injected stream write failure"))
        } else {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::other("injected stream flush failure"))
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
#[allow(dead_code)]
struct FailThirdFlush {
    flushes: usize,
}

impl Write for FailThirdFlush {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let flush = self.flushes;
        self.flushes += 1;
        if flush == 2 {
            Err(io::Error::other("injected CSV finish flush failure"))
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
#[allow(dead_code)]
struct FailSecondFlush {
    flushes: usize,
}

impl Write for FailSecondFlush {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let flush = self.flushes;
        self.flushes += 1;
        if flush == 1 {
            Err(io::Error::other("injected CSV into-inner failure"))
        } else {
            Ok(())
        }
    }
}

#[allow(dead_code)]
struct LimitedCursor {
    inner: Cursor<Vec<u8>>,
    max_len: u64,
}

impl LimitedCursor {
    #[allow(dead_code)]
    const fn new(max_len: u64) -> Self {
        Self {
            inner: Cursor::new(Vec::new()),
            max_len,
        }
    }
}

impl std::io::Read for LimitedCursor {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buffer)
    }
}

impl Write for LimitedCursor {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let end = self
            .inner
            .position()
            .saturating_add(u64::try_from(buffer.len()).unwrap_or(u64::MAX));
        if end > self.max_len {
            return Err(io::Error::other("injected encrypted output failure"));
        }
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

#[allow(dead_code)]
struct ToggleFlushFailure {
    fail: Arc<AtomicBool>,
}

impl Write for ToggleFlushFailure {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.fail.load(Ordering::SeqCst) {
            Err(io::Error::other("injected close failure"))
        } else {
            Ok(())
        }
    }
}

#[allow(dead_code)]
struct EnableFlushFailure(Arc<AtomicBool>);

impl WriteHandler for EnableFlushFailure {
    fn after_workbook(&mut self, _context: &WriteWorkbookContext) -> Result<()> {
        self.0.store(true, Ordering::SeqCst);
        Ok(())
    }
}

impl Seek for LimitedCursor {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.inner.seek(position)
    }
}

#[allow(dead_code)]
fn zip_entry(path: &Path, name: &str) -> Result<String> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(test_error)?;
    let mut entry = archive.by_name(name).map_err(test_error)?;
    let mut value = String::new();
    entry.read_to_string(&mut value)?;
    Ok(value)
}

#[allow(dead_code)]
fn zip_names(path: &Path) -> Result<Vec<String>> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file).map_err(test_error)?;
    (0..archive.len())
        .map(|index| {
            archive
                .by_index(index)
                .map(|entry| entry.name().to_owned())
                .map_err(test_error)
        })
        .collect::<Result<Vec<_>>>()
}

#[allow(dead_code)]
fn cell_style_id(sheet_xml: &str, cell: &str) -> Option<String> {
    let marker = format!("<c r=\"{cell}\" s=\"");
    sheet_xml
        .split_once(&marker)
        .and_then(|(_, value)| value.split_once('"'))
        .map(|(style, _)| style.to_owned())
}

#[allow(dead_code)]
fn sheet_column_width(sheet_xml: &str, one_based_column: u16) -> Result<f64> {
    let marker = format!("<col min=\"{one_based_column}\"");
    let (_, column) = sheet_xml
        .split_once(&marker)
        .ok_or_else(|| test_error(format!("missing column {one_based_column}")))?;
    let (_, width) = column
        .split_once("width=\"")
        .ok_or_else(|| test_error("missing column width"))?;
    let (width, _) = width
        .split_once('"')
        .ok_or_else(|| test_error("unterminated column width"))?;
    width.parse::<f64>().map_err(test_error)
}

#[allow(dead_code)]
fn sheet_row_height(sheet_xml: &str, one_based_row: u32) -> Result<f64> {
    let marker = format!("<row r=\"{one_based_row}\"");
    let (_, row) = sheet_xml
        .split_once(&marker)
        .ok_or_else(|| test_error(format!("missing row {one_based_row}")))?;
    let (row, _) = row
        .split_once('>')
        .ok_or_else(|| test_error("unterminated row"))?;
    let (_, height) = row
        .split_once("ht=\"")
        .ok_or_else(|| test_error("missing row height"))?;
    let (height, _) = height
        .split_once('"')
        .ok_or_else(|| test_error("unterminated row height"))?;
    height.parse::<f64>().map_err(test_error)
}

#[derive(Clone)]
#[allow(dead_code)]
struct EveryCell {
    cells: Vec<CellValue>,
    fail: bool,
}

thread_local! {
    static USE_WIDE_SCHEMA: Cell<bool> = const { Cell::new(false) };
    static USE_ANNOTATED_WIDE_SCHEMA: Cell<bool> = const { Cell::new(false) };
    static USE_BACKEND_WIDE_SCHEMA: Cell<bool> = const { Cell::new(false) };
}

#[allow(dead_code)]
const TEST_COLUMN: ExcelColumn = ExcelColumn::new("value", "Value", Some(0), 0, None);

#[allow(dead_code)]
struct SparseRow;

#[allow(dead_code)]
struct DimensionRow;

#[allow(dead_code)]
struct StyledAnnotationRow;

#[allow(dead_code)]
struct AnnotationHandlerRow(&'static str);

impl ExcelRow for AnnotationHandlerRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)
            .with_column_width(18)
            .with_loop_merge(crate::core::LoopMergeProperty::new(2, 1))];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
            .head_row_height(31)
            .content_row_height(24)
            .once_absolute_merge(OnceAbsoluteMergeProperty::new(10, 10, 0, 1));
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self(""))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::String(self.0.to_owned())])
    }
}

impl ExcelRow for StyledAnnotationRow {
    fn schema() -> &'static [ExcelColumn] {
        const FIELD_HEAD_STYLE: ExcelCellStyle = ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Indexed(14)),
            horizontal_alignment: Some(ExcelHorizontalAlignment::Left),
            ..ExcelCellStyle::new()
        };
        const FIELD_HEAD_FONT: ExcelFontStyle = ExcelFontStyle {
            font_height_in_points: Some(40.0),
            color: Some(ExcelColor::Indexed(51)),
            ..ExcelFontStyle::new()
        };
        const FIELD_CONTENT_STYLE: ExcelCellStyle = ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Indexed(40)),
            ..ExcelCellStyle::new()
        };
        const FIELD_CONTENT_FONT: ExcelFontStyle = ExcelFontStyle {
            font_height_in_points: Some(50.0),
            color: Some(ExcelColor::Indexed(12)),
            ..ExcelFontStyle::new()
        };
        const COLUMNS: &[ExcelColumn] = &[
            ExcelColumn::new("field", "Field", Some(0), 0, None)
                .with_head_style(FIELD_HEAD_STYLE)
                .with_head_font_style(FIELD_HEAD_FONT)
                .with_content_style(FIELD_CONTENT_STYLE)
                .with_content_font_style(FIELD_CONTENT_FONT),
            ExcelColumn::new("type", "Type", Some(1), 0, None),
        ];
        COLUMNS
    }

    fn write_metadata() -> &'static ExcelWriteMetadata {
        const HEAD_STYLE: ExcelCellStyle = ExcelCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Indexed(10)),
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..ExcelCellStyle::new()
        };
        const CONTENT_STYLE: ExcelCellStyle = ExcelCellStyle {
            border_bottom: Some(ExcelBorderStyle::Thin),
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_foreground_color: Some(ExcelColor::Indexed(17)),
            ..ExcelCellStyle::new()
        };
        const HEAD_FONT: ExcelFontStyle = ExcelFontStyle {
            bold: Some(false),
            font_height_in_points: Some(20.0),
            color: Some(ExcelColor::Indexed(15)),
            ..ExcelFontStyle::new()
        };
        const CONTENT_FONT: ExcelFontStyle = ExcelFontStyle {
            font_height_in_points: Some(30.0),
            color: Some(ExcelColor::Indexed(22)),
            ..ExcelFontStyle::new()
        };
        const METADATA: ExcelWriteMetadata = ExcelWriteMetadata::new()
            .head_style(HEAD_STYLE)
            .content_style(CONTENT_STYLE)
            .head_font_style(HEAD_FONT)
            .content_font_style(CONTENT_FONT);
        &METADATA
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self)
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![
            CellValue::String("field".to_owned()),
            CellValue::String("type".to_owned()),
        ])
    }
}

#[allow(dead_code)]
struct OverrideAnnotationDimensions;

impl WriteHandler for OverrideAnnotationDimensions {
    fn order(&self) -> i32 {
        crate::constant::order_constant::DEFINE_STYLE
    }

    fn style_column_width(&self, _column_index: usize) -> Option<u16> {
        Some(27)
    }

    fn style_head_row_height(&self) -> Option<u16> {
        Some(40)
    }

    fn style_content_row_height(&self) -> Option<u16> {
        Some(36)
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct ContextStringConverter(&'static str);

impl Converter<String> for ContextStringConverter {
    fn convert_to_excel_data(
        &self,
        context: &crate::core::WriteConverterContext<'_, String>,
    ) -> Result<WriteCellData> {
        Ok(WriteCellData::from_string(format!(
            "{}:{}",
            self.0,
            context.value()
        )))
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
struct ContextI32Converter(&'static str);

impl Converter<i32> for ContextI32Converter {
    fn convert_to_excel_data(
        &self,
        context: &crate::core::WriteConverterContext<'_, i32>,
    ) -> Result<WriteCellData> {
        Ok(WriteCellData::from_string(format!(
            "{}:{}",
            self.0,
            context.value()
        )))
    }
}

#[allow(dead_code)]
struct ConverterMapProbe(Arc<Mutex<Vec<(String, String)>>>);

impl WriteHandler for ConverterMapProbe {
    fn after_cell_data_converted(&mut self, context: &WriteCellContext) -> Result<()> {
        if context.is_head {
            return Ok(());
        }
        let registry = context
            .write_context()
            .current_write_holder()
            .converter_map();
        let column = ExcelColumn::new("value", "Value", Some(0), 0, None);
        let convert_context = crate::core::ConvertContext {
            sheet_name: context.sheet_name.clone(),
            row_index: context.row_index,
            column_index: Some(usize::from(context.column_index)),
            field: "value",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        };
        let string_value = registry
            .convert_to_excel_data(&"probe".to_owned(), &column, &convert_context)?
            .expect("effective holder must contain the sheet/table String converter")
            .value()
            .as_text();
        let integer_value = registry
            .convert_to_excel_data(&7_i32, &column, &convert_context)?
            .expect("effective holder must inherit the workbook i32 converter")
            .value()
            .as_text();
        self.0
            .lock()
            .map_err(|_| ExcelError::Format("converter map probe poisoned".to_owned()))?
            .push((string_value, integer_value));
        Ok(())
    }
}

#[allow(dead_code)]
struct ConverterContextRow(String);

impl ExcelRow for ConverterContextRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self(String::new()))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::String(self.0.clone())])
    }

    fn to_excel_write_row(
        &self,
        converters: &ConverterRegistry,
    ) -> Result<(Vec<CellValue>, Vec<WriteCellData>)> {
        let original = CellValue::String(self.0.clone());
        let converted = converters
            .convert_to_excel_data(
                &self.0,
                &Self::schema()[0],
                &crate::core::ConvertContext {
                    sheet_name: String::new(),
                    row_index: 0,
                    column_index: Some(0),
                    field: "value",
                    format: None,
                    date_time_format: None,
                    number_format: None,
                    use_1904_windowing: false,
                },
            )?
            .unwrap_or_else(|| WriteCellData::new(original.clone()));
        Ok((vec![original], vec![converted]))
    }
}

#[allow(dead_code)]
struct NumericConverterContextRow(i32);

impl ExcelRow for NumericConverterContextRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self(0))
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::Int(i64::from(self.0))])
    }

    fn to_excel_write_row(
        &self,
        converters: &ConverterRegistry,
    ) -> Result<(Vec<CellValue>, Vec<WriteCellData>)> {
        let original = CellValue::Int(i64::from(self.0));
        let converted = converters
            .convert_to_excel_data(
                &self.0,
                &Self::schema()[0],
                &crate::core::ConvertContext {
                    sheet_name: String::new(),
                    row_index: 0,
                    column_index: Some(0),
                    field: "value",
                    format: None,
                    date_time_format: None,
                    number_format: None,
                    use_1904_windowing: false,
                },
            )?
            .unwrap_or_else(|| WriteCellData::new(original.clone()));
        Ok((vec![original], vec![converted]))
    }
}

#[allow(dead_code)]
struct DefaultRegistryRequiredRow;

impl ExcelRow for DefaultRegistryRequiredRow {
    fn schema() -> &'static [ExcelColumn] {
        const COLUMNS: &[ExcelColumn] = &[ExcelColumn::new("value", "Value", Some(0), 0, None)];
        COLUMNS
    }

    fn from_row(_row: &crate::core::RowData) -> Result<Self> {
        Ok(Self)
    }

    fn to_row(&self) -> Result<Vec<CellValue>> {
        Ok(vec![CellValue::Int(7)])
    }

    fn to_row_with_converters(&self, converters: &ConverterRegistry) -> Result<Vec<CellValue>> {
        if converters.is_empty() {
            return Err(test_error("default write converter registry is missing"));
        }
        self.to_row()
    }
}

#[allow(dead_code)]
struct ConvertedTypeProbe(Arc<Mutex<Vec<crate::core::CellDataType>>>);

impl WriteHandler for ConvertedTypeProbe {
    fn after_cell_data_converted(&mut self, context: &WriteCellContext) -> Result<()> {
        if !context.is_head
            && let Some(value) = context.first_cell_data()
        {
            self.0
                .lock()
                .map_err(|_| test_error("converted type probe poisoned"))?
                .push(value.data_type());
        }
        Ok(())
    }
}

/// Compute SHA-256 hex digest of a file on disk.
#[allow(dead_code)]
fn sha256_of_file(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(path).expect("read file for sha256");
    let hash = Sha256::digest(&bytes);
    format!("{hash:x}")
}

/// Assert that all given file paths produce the same SHA-256 checksum.
#[allow(dead_code)]
fn assert_same_checksum(paths: &[&Path]) {
    assert!(paths.len() >= 2, "need at least 2 paths to compare");
    let first = sha256_of_file(paths[0]);
    for path in &paths[1..] {
        let current = sha256_of_file(path);
        assert_eq!(
            first,
            current,
            "checksum mismatch: {} vs {}",
            paths[0].display(),
            path.display()
        );
    }
}

include!("tests_cases/cases_01.rs");
include!("tests_cases/cases_02.rs");
include!("tests_cases/cases_03.rs");
include!("tests_cases/cases_04.rs");
include!("tests_cases/cases_05.rs");
include!("tests_cases/cases_06.rs");
include!("tests_cases/cases_07.rs");
include!("tests_cases/cases_08.rs");
include!("tests_cases/cases_09.rs");
include!("tests_cases/cases_10.rs");
include!("tests_cases/cases_11.rs");
include!("tests_cases/cases_12_spill_matrix.rs");
