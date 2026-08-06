use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::core::{CellValue, ExcelError, Result};
use crate::write::ExcelOutputStream;
use ::bigdecimal::BigDecimal;
use base64::Engine;
use calamine::{Data, Reader, Xlsx, open_workbook};
use chrono::NaiveDate;
use flate2::read::GzDecoder;
use num_bigint::BigInt;
use rust_xlsxwriter::{Format, Workbook};
use tempfile::{TempDir, tempdir};
use zip::CompressionMethod;

use super::*;
use crate::template::fill_engine::*;
use crate::template::template_entry::*;
use crate::template::template_output::*;
use crate::template::template_writer::*;

struct FaultyIo {
    inner: Cursor<Vec<u8>>,
    fail_read_at: Option<usize>,
    fail_write_at: Option<usize>,
    fail_seek_at: Option<usize>,
    reads: usize,
    writes: usize,
    seeks: usize,
}

#[derive(Debug, Default)]
struct SharedOutputState {
    bytes: Vec<u8>,
    fail_write: bool,
    fail_flush: bool,
    flushes: usize,
}

#[derive(Clone, Debug)]
struct SharedOutput(Arc<Mutex<SharedOutputState>>);

impl SharedOutput {
    fn new(fail_write: bool, fail_flush: bool) -> Self {
        Self(Arc::new(Mutex::new(SharedOutputState {
            fail_write,
            fail_flush,
            ..SharedOutputState::default()
        })))
    }
}

impl Write for SharedOutput {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let mut state = self.0.lock().expect("output state lock");
        if state.fail_write {
            return Err(io::Error::other("injected stream write failure"));
        }
        state.bytes.extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut state = self.0.lock().expect("output state lock");
        state.flushes += 1;
        if state.fail_flush {
            Err(io::Error::other("injected stream flush failure"))
        } else {
            Ok(())
        }
    }
}

struct DropReader {
    inner: Cursor<Vec<u8>>,
    dropped: Arc<AtomicBool>,
}

impl DropReader {
    fn new(bytes: Vec<u8>, dropped: Arc<AtomicBool>) -> Self {
        Self {
            inner: Cursor::new(bytes),
            dropped,
        }
    }
}

impl Read for DropReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buffer)
    }
}

impl Drop for DropReader {
    fn drop(&mut self) {
        self.dropped.store(true, Ordering::SeqCst);
    }
}

impl FaultyIo {
    fn reading(bytes: Vec<u8>, fail_at: usize) -> Self {
        Self {
            inner: Cursor::new(bytes),
            fail_read_at: Some(fail_at),
            fail_write_at: None,
            fail_seek_at: None,
            reads: 0,
            writes: 0,
            seeks: 0,
        }
    }

    fn writing(fail_at: usize) -> Self {
        Self {
            inner: Cursor::new(Vec::new()),
            fail_read_at: None,
            fail_write_at: Some(fail_at),
            fail_seek_at: None,
            reads: 0,
            writes: 0,
            seeks: 0,
        }
    }
}

impl Read for FaultyIo {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let call = self.reads;
        self.reads += 1;
        if self.fail_read_at == Some(call) {
            return Err(io::Error::other("injected read failure"));
        }
        self.inner.read(buffer)
    }
}

impl Write for FaultyIo {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let call = self.writes;
        self.writes += 1;
        if self.fail_write_at == Some(call) {
            return Err(io::Error::other("injected write failure"));
        }
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl Seek for FaultyIo {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let call = self.seeks;
        self.seeks += 1;
        if self.fail_seek_at == Some(call) {
            return Err(io::Error::other("injected seek failure"));
        }
        self.inner.seek(position)
    }
}

fn test_error(error: impl std::fmt::Display) -> ExcelError {
    ExcelError::Format(error.to_string())
}

fn template_fixture() -> Result<(TempDir, std::path::PathBuf)> {
    let directory = tempdir()?;
    let path = directory.path().join("template.xlsx");
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .write_string(0, 0, "Hello {name}")
        .map_err(test_error)?;
    worksheet
        .write_string(1, 0, "Count: {count}")
        .map_err(test_error)?;
    worksheet
        .write_string(2, 0, "Unknown: {unknown}")
        .map_err(test_error)?;
    workbook.save(&path).map_err(test_error)?;
    Ok((directory, path))
}

fn multi_sheet_template_fixture() -> Result<(TempDir, std::path::PathBuf)> {
    let directory = tempdir()?;
    let path = directory.path().join("multi-sheet-template.xlsx");
    let mut workbook = Workbook::new();
    let summary = workbook.add_worksheet();
    summary.set_name("摘要").map_err(test_error)?;
    summary.write_string(0, 0, "{title}").map_err(test_error)?;

    let details = workbook.add_worksheet();
    details.set_name("明细").map_err(test_error)?;
    details.write_string(0, 0, "{title}").map_err(test_error)?;
    details
        .write_string(1, 0, "{items.name}")
        .map_err(test_error)?;
    details
        .write_string(1, 1, "{items.value}")
        .map_err(test_error)?;

    let untouched = workbook.add_worksheet();
    untouched.set_name("未处理").map_err(test_error)?;
    untouched
        .write_string(0, 0, "{title}")
        .map_err(test_error)?;
    workbook.save(&path).map_err(test_error)?;
    Ok((directory, path))
}

fn write_compressed_java_fixture(path: &Path, fixture: &str) -> Result<()> {
    let compressed = base64::engine::general_purpose::STANDARD
        .decode(fixture.trim())
        .map_err(test_error)?;
    let mut decoder = GzDecoder::new(compressed.as_slice());
    let mut workbook = Vec::new();
    decoder.read_to_end(&mut workbook)?;
    fs::write(path, workbook)?;
    Ok(())
}

fn write_java_composite_fixture(path: &Path) -> Result<()> {
    write_compressed_java_fixture(
        path,
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-demo-composite.xlsx.gz.b64"
        )),
    )
}

fn synthetic_entry(name: &str, bytes: impl Into<Vec<u8>>) -> TemplateEntry {
    TemplateEntry {
        name: name.to_owned(),
        is_dir: false,
        compression: CompressionMethod::Stored,
        unix_mode: None,
        bytes: bytes.into(),
    }
}

fn find_string_coordinate(range: &calamine::Range<Data>, needle: &str) -> Option<(u32, u32)> {
    range.cells().find_map(|(row, column, value)| {
        (value == &Data::String(needle.to_owned())).then(|| {
            (
                u32::try_from(row).expect("small row"),
                u32::try_from(column).expect("small column"),
            )
        })
    })
}

include!("tests_cases/cases_01.rs");
include!("tests_cases/cases_02.rs");
include!("tests_cases/cases_03.rs");
