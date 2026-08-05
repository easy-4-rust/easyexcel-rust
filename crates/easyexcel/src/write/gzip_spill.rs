//! SXSSF `GZIPSheetDataWriter` equivalent: gzip-compressed row spill on disk.
//!
//! Java `SXSSFWorkbook.setCompressTempFiles(true)` routes sheet XML through
//! `GZIPSheetDataWriter`. `rust_xlsxwriter` constant-memory mode cannot gzip its
//! internal tempfile, so this module owns the durable spill while rows are
//! written; [`ExcelWriter`] materializes into a constant-memory worksheet only
//! at `finish` (stream decode → write → ZIP), keeping peak RAM bounded.

use std::path::{Path, PathBuf};

use crate::core::{CellValue, ExcelError, ImageData, Result, RichTextStringData};
use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use easyexcel_io::io::gzip_record::{GzipRecordReader, GzipRecordWriter};

pub use easyexcel_io::io::gzip_record::{GZIP_MAGIC, file_has_gzip_magic};

/// Observable snapshot of an active or finished gzip spill file.
#[derive(Debug, Clone)]
pub struct GzipSpillSnapshot {
    /// Logical sheet name this spill belongs to.
    pub sheet_name: String,
    /// Path of the gzip tempfile (named, so tests can open it).
    pub path: PathBuf,
    /// Whether the file begins with gzip magic.
    pub is_gzip: bool,
    /// On-disk compressed size in bytes.
    pub compressed_len: u64,
    /// Uncompressed payload bytes written into the encoder.
    pub uncompressed_len: u64,
}

/// Streaming gzip spill writer mirroring POI `GZIPSheetDataWriter`.
pub struct GzipSheetDataWriter {
    sheet_name: String,
    writer: GzipRecordWriter,
}

impl GzipSheetDataWriter {
    /// Creates a new gzip spill file under `dir` for `sheet_name`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the tempfile cannot be created.
    pub fn create(dir: &Path, sheet_name: impl Into<String>) -> Result<Self> {
        let sheet_name = sheet_name.into();
        Ok(Self {
            sheet_name,
            writer: GzipRecordWriter::create(dir, "easyexcel-sxssf-", ".xml.gz")
                .map_err(ExcelError::from)?,
        })
    }

    /// Creates a spill that owns its temporary directory (deleted on drop).
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the temp directory or file cannot be created.
    pub fn create_owned(sheet_name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            sheet_name: sheet_name.into(),
            writer: GzipRecordWriter::create_owned("easyexcel-sxssf-", ".xml.gz")
                .map_err(ExcelError::from)?,
        })
    }

    /// Appends one data row (cell values) to the gzip spill.
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when encoding or writing fails.
    pub fn write_row(&mut self, cells: &[CellValue]) -> Result<()> {
        let payload = encode_row(cells)?;
        self.writer.write_record(&payload).map_err(ExcelError::from)
    }

    /// Flushes buffered gzip bytes so magic / size are observable on disk.
    ///
    /// # Errors
    ///
    /// Returns an I/O error on flush failure.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(ExcelError::from)
    }

    /// Returns a snapshot suitable for tests (gzip magic + sizes).
    ///
    /// # Errors
    ///
    /// Returns an I/O error when flushing or stating the file fails.
    pub fn snapshot(&mut self) -> Result<GzipSpillSnapshot> {
        let snapshot = self.writer.snapshot().map_err(ExcelError::from)?;
        Ok(GzipSpillSnapshot {
            sheet_name: self.sheet_name.clone(),
            path: snapshot.path,
            is_gzip: snapshot.is_gzip,
            compressed_len: snapshot.compressed_len,
            uncompressed_len: snapshot.uncompressed_len,
        })
    }

    /// Finishes the encoder and returns a readable spill handle.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when finishing gzip or reopening the file fails.
    pub fn finish(self) -> Result<GzipSpillReader> {
        Ok(GzipSpillReader {
            sheet_name: self.sheet_name,
            reader: self.writer.finish().map_err(ExcelError::from)?,
        })
    }
}

/// Read side of a finished gzip spill (stream decode, constant memory).
pub struct GzipSpillReader {
    sheet_name: String,
    reader: GzipRecordReader,
}

impl GzipSpillReader {
    /// Returns spill metadata after finish.
    #[must_use]
    pub fn snapshot(&self) -> GzipSpillSnapshot {
        let snapshot = self.reader.snapshot();
        GzipSpillSnapshot {
            sheet_name: self.sheet_name.clone(),
            path: snapshot.path,
            is_gzip: snapshot.is_gzip,
            compressed_len: snapshot.compressed_len,
            uncompressed_len: snapshot.uncompressed_len,
        }
    }

    /// Decodes the next spilled row, or `None` at EOF.
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when the stream is corrupt.
    pub fn next_row(&mut self) -> Result<Option<Vec<CellValue>>> {
        self.reader
            .next_record()
            .map_err(ExcelError::from)?
            .map(|payload| decode_row(&payload))
            .transpose()
    }
}

fn encode_row(cells: &[CellValue]) -> Result<Vec<u8>> {
    let mut body = Vec::with_capacity(cells.len() * 16);
    write_u32(
        &mut body,
        u32::try_from(cells.len())
            .map_err(|_| ExcelError::Format("row cell count exceeds u32".to_owned()))?,
    );
    for cell in cells {
        encode_cell(&mut body, cell)?;
    }
    Ok(body)
}

fn decode_row(payload: &[u8]) -> Result<Vec<CellValue>> {
    let mut cursor = 0usize;
    let count = read_u32(payload, &mut cursor)? as usize;
    let mut cells = Vec::with_capacity(count);
    for _ in 0..count {
        cells.push(decode_cell(payload, &mut cursor)?);
    }
    Ok(cells)
}

fn encode_cell(out: &mut Vec<u8>, value: &CellValue) -> Result<()> {
    match value {
        CellValue::Empty => out.push(0),
        CellValue::String(text) => {
            out.push(1);
            write_str(out, text)?;
        }
        CellValue::Bool(flag) => {
            out.push(2);
            out.push(u8::from(*flag));
        }
        CellValue::Int(number) => {
            out.push(3);
            out.extend_from_slice(&number.to_le_bytes());
        }
        CellValue::Float(number) => {
            out.push(4);
            out.extend_from_slice(&number.to_le_bytes());
        }
        CellValue::Decimal(number) => {
            out.push(5);
            write_str(out, &number.to_string())?;
        }
        CellValue::Date(date) => {
            out.push(6);
            write_str(out, &date.format("%Y-%m-%d").to_string())?;
        }
        CellValue::DateTime(date_time) => {
            out.push(7);
            write_str(out, &date_time.format("%Y-%m-%d %H:%M:%S%.f").to_string())?;
        }
        CellValue::Error(text) => {
            out.push(8);
            write_str(out, text)?;
        }
        CellValue::Formula(text) => {
            out.push(9);
            write_str(out, text)?;
        }
        CellValue::Hyperlink { url, text } => {
            out.push(10);
            write_str(out, url)?;
            write_str(out, text)?;
        }
        CellValue::Comment { value, text } => {
            out.push(11);
            write_str(out, text)?;
            encode_cell(out, value)?;
        }
        CellValue::Image(bytes) => {
            out.push(12);
            write_bytes(out, bytes)?;
        }
        CellValue::RichText(rich) => {
            // Fonts are not required for compress-temp spill round-trips.
            out.push(13);
            write_str(out, rich.text_string())?;
        }
        CellValue::Images { value, images } => {
            out.push(14);
            encode_cell(out, value)?;
            write_u32(
                out,
                u32::try_from(images.len())
                    .map_err(|_| ExcelError::Format("image list exceeds u32".to_owned()))?,
            );
            for image in images {
                write_bytes(out, image.image())?;
            }
        }
    }
    Ok(())
}

fn decode_cell(buf: &[u8], cursor: &mut usize) -> Result<CellValue> {
    let tag = read_u8(buf, cursor)?;
    Ok(match tag {
        0 => CellValue::Empty,
        1 => CellValue::String(read_str(buf, cursor)?),
        2 => CellValue::Bool(read_u8(buf, cursor)? != 0),
        3 => {
            let bytes = read_exact::<8>(buf, cursor)?;
            CellValue::Int(i64::from_le_bytes(bytes))
        }
        4 => {
            let bytes = read_exact::<8>(buf, cursor)?;
            CellValue::Float(f64::from_le_bytes(bytes))
        }
        5 => {
            let text = read_str(buf, cursor)?;
            let number: BigDecimal = text
                .parse()
                .map_err(|error| ExcelError::Format(format!("invalid decimal spill: {error}")))?;
            CellValue::Decimal(number)
        }
        6 => {
            let text = read_str(buf, cursor)?;
            let date = NaiveDate::parse_from_str(&text, "%Y-%m-%d")
                .map_err(|error| ExcelError::Format(format!("invalid date spill: {error}")))?;
            CellValue::Date(date)
        }
        7 => {
            let text = read_str(buf, cursor)?;
            let date_time = NaiveDateTime::parse_from_str(&text, "%Y-%m-%d %H:%M:%S%.f")
                .or_else(|_| NaiveDateTime::parse_from_str(&text, "%Y-%m-%d %H:%M:%S"))
                .map_err(|error| ExcelError::Format(format!("invalid datetime spill: {error}")))?;
            CellValue::DateTime(date_time)
        }
        8 => CellValue::Error(read_str(buf, cursor)?),
        9 => CellValue::Formula(read_str(buf, cursor)?),
        10 => CellValue::Hyperlink {
            url: read_str(buf, cursor)?,
            text: read_str(buf, cursor)?,
        },
        11 => {
            let text = read_str(buf, cursor)?;
            let value = Box::new(decode_cell(buf, cursor)?);
            CellValue::Comment { value, text }
        }
        12 => CellValue::Image(read_bytes(buf, cursor)?),
        13 => CellValue::RichText(RichTextStringData::new(read_str(buf, cursor)?)),
        14 => {
            let value = Box::new(decode_cell(buf, cursor)?);
            let count = read_u32(buf, cursor)? as usize;
            let mut images = Vec::with_capacity(count);
            for _ in 0..count {
                images.push(ImageData::new(read_bytes(buf, cursor)?));
            }
            CellValue::Images { value, images }
        }
        other => {
            return Err(ExcelError::Format(format!(
                "unknown gzip spill cell tag: {other}"
            )));
        }
    })
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_str(out: &mut Vec<u8>, value: &str) -> Result<()> {
    write_bytes(out, value.as_bytes())
}

fn write_bytes(out: &mut Vec<u8>, value: &[u8]) -> Result<()> {
    write_u32(
        out,
        u32::try_from(value.len())
            .map_err(|_| ExcelError::Format("spill byte length exceeds u32".to_owned()))?,
    );
    out.extend_from_slice(value);
    Ok(())
}

fn read_u8(buf: &[u8], cursor: &mut usize) -> Result<u8> {
    let value = *buf
        .get(*cursor)
        .ok_or_else(|| ExcelError::Format("gzip spill truncated (u8)".to_owned()))?;
    *cursor += 1;
    Ok(value)
}

fn read_u32(buf: &[u8], cursor: &mut usize) -> Result<u32> {
    let bytes = read_exact::<4>(buf, cursor)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_exact<const N: usize>(buf: &[u8], cursor: &mut usize) -> Result<[u8; N]> {
    let end = cursor
        .checked_add(N)
        .ok_or_else(|| ExcelError::Format("gzip spill cursor overflow".to_owned()))?;
    let slice = buf
        .get(*cursor..end)
        .ok_or_else(|| ExcelError::Format("gzip spill truncated".to_owned()))?;
    let mut out = [0u8; N];
    out.copy_from_slice(slice);
    *cursor = end;
    Ok(out)
}

fn read_str(buf: &[u8], cursor: &mut usize) -> Result<String> {
    let bytes = read_bytes(buf, cursor)?;
    String::from_utf8(bytes)
        .map_err(|error| ExcelError::Format(format!("gzip spill utf-8: {error}")))
}

fn read_bytes(buf: &[u8], cursor: &mut usize) -> Result<Vec<u8>> {
    let len = read_u32(buf, cursor)? as usize;
    let end = cursor
        .checked_add(len)
        .ok_or_else(|| ExcelError::Format("gzip spill cursor overflow".to_owned()))?;
    let slice = buf
        .get(*cursor..end)
        .ok_or_else(|| ExcelError::Format("gzip spill truncated (bytes)".to_owned()))?;
    *cursor = end;
    Ok(slice.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn gzip_spill_round_trips_cells_and_exposes_magic() {
        let mut writer = GzipSheetDataWriter::create_owned("Sheet1").expect("create");
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).expect("date");
        writer
            .write_row(&[
                CellValue::String("字符串0".to_owned()),
                CellValue::Date(date),
                CellValue::Float(0.56),
                CellValue::Int(42),
                CellValue::Bool(true),
                CellValue::Empty,
            ])
            .expect("write");
        let snap = writer.snapshot().expect("snapshot");
        assert!(snap.is_gzip, "spill must start with gzip magic");
        assert!(snap.uncompressed_len > 0);
        assert!(snap.compressed_len > 0);
        // Highly repetitive / small payloads may not shrink, but magic must be present.
        assert_eq!(&snap.path.extension().and_then(|e| e.to_str()), &Some("gz"));

        let mut reader = writer.finish().expect("finish");
        let row = reader.next_row().expect("decode").expect("one row");
        assert_eq!(row[0], CellValue::String("字符串0".to_owned()));
        assert_eq!(row[1], CellValue::Date(date));
        assert!(matches!(row[2], CellValue::Float(v) if (v - 0.56).abs() < f64::EPSILON));
        assert_eq!(row[3], CellValue::Int(42));
        assert_eq!(row[4], CellValue::Bool(true));
        assert_eq!(row[5], CellValue::Empty);
        assert!(reader.next_row().expect("eof").is_none());
        assert!(reader.snapshot().is_gzip);
    }

    #[test]
    fn gzip_spill_round_trips_every_cell_value_variant() {
        let mut writer = GzipSheetDataWriter::create_owned("All").expect("create");
        let date = NaiveDate::from_ymd_opt(2020, 1, 1).expect("date");
        let datetime = date.and_hms_opt(12, 30, 0).expect("datetime");
        let datetime_nanos = date
            .and_hms_nano_opt(12, 30, 0, 123_000_000)
            .expect("datetime nanos");
        let rows = vec![
            CellValue::Empty,
            CellValue::String("s".to_owned()),
            CellValue::Bool(false),
            CellValue::Int(-7),
            CellValue::Float(1.25),
            CellValue::Decimal(BigDecimal::from(3)),
            CellValue::Date(date),
            CellValue::DateTime(datetime),
            CellValue::DateTime(datetime_nanos),
            CellValue::Error("#DIV/0!".to_owned()),
            CellValue::Formula("B2+C2".to_owned()),
            CellValue::Hyperlink {
                url: "https://x".to_owned(),
                text: "link".to_owned(),
            },
            CellValue::Comment {
                value: Box::new(CellValue::Bool(true)),
                text: "note".to_owned(),
            },
            CellValue::Image(vec![1, 2, 3]),
            CellValue::RichText(RichTextStringData::new("rich")),
            CellValue::Images {
                value: Box::new(CellValue::Int(5)),
                images: vec![ImageData::new(vec![9, 8])],
            },
        ];
        writer.write_row(&rows).expect("write");
        let mut reader = writer.finish().expect("finish");
        let decoded = reader.next_row().expect("decode").expect("row");
        assert_eq!(decoded, rows);
        assert!(reader.next_row().expect("eof").is_none());
    }

    #[test]
    fn gzip_spill_decode_reports_corrupt_payloads() {
        let mut cursor = 0usize;
        // Decimal parse failure.
        let decimal_err = decode_cell(&[5, 3, 0, 0, 0, b'a', b'b', b'c'], &mut cursor)
            .expect_err("invalid decimal must fail");
        assert!(matches!(decimal_err, ExcelError::Format(_)));
        // Date parse failure.
        cursor = 0;
        let date_err = decode_cell(&[6, 3, 0, 0, 0, b'b', b'a', b'd'], &mut cursor)
            .expect_err("invalid date must fail");
        assert!(matches!(date_err, ExcelError::Format(_)));
        // DateTime parse failure (both fallback formats fail).
        cursor = 0;
        let datetime_err = decode_cell(&[7, 3, 0, 0, 0, b'b', b'a', b'd'], &mut cursor)
            .expect_err("invalid datetime must fail");
        assert!(matches!(datetime_err, ExcelError::Format(_)));
        // Unknown tag.
        cursor = 0;
        let unknown = decode_cell(&[99], &mut cursor).expect_err("unknown tag must fail");
        assert!(matches!(unknown, ExcelError::Format(_)));
        // String payload shorter than its declared length.
        cursor = 0;
        let truncated =
            decode_cell(&[1, 10, 0, 0, 0], &mut cursor).expect_err("truncated payload must fail");
        assert!(matches!(truncated, ExcelError::Format(_)));
        // Non-UTF-8 string payload.
        cursor = 0;
        let invalid_utf8 =
            decode_cell(&[8, 1, 0, 0, 0, 0xFF], &mut cursor).expect_err("invalid UTF-8 must fail");
        assert!(matches!(invalid_utf8, ExcelError::Format(_)));
    }

    #[test]
    fn gzip_spill_reader_reports_non_eof_stream_errors() {
        let directory = tempfile::tempdir().expect("tempdir");
        let bad_path = directory.path().join("bad.gz");
        std::fs::write(&bad_path, b"not a gzip stream").expect("write");
        let mut reader = GzipSpillReader {
            sheet_name: "Sheet1".to_owned(),
            reader: GzipRecordReader::open_path(bad_path).expect("open"),
        };
        let error = reader.next_row().expect_err("corrupt stream must fail");
        assert!(matches!(error, ExcelError::Io(_)));
    }

    #[test]
    fn file_has_gzip_magic_missing_file_returns_false() {
        assert!(!file_has_gzip_magic(std::path::Path::new(
            "/nonexistent/path/definitely-missing.gz"
        )));
    }
}
