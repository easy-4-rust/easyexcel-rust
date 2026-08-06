//! CSV import/export. Maps a delimited text file to/from a single-sheet
//! [`Workbook`], with delimiter auto-detection, BOM handling, non-UTF-8
//! transcoding via `encoding_rs`, and import-time type inference.

use std::io::{Read, Write};

use chrono::NaiveDate;
use encoding_rs::Encoding;
use encoding_rs_io::{DecodeReaderBytes, DecodeReaderBytesBuilder};

use easyexcel_io::{Error, Result};
use easyexcel_model::{Cell, Sheet, Workbook};

use super::CsvCharset;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Resolve a Java/WHATWG charset label to the encoding used by the CSV engine.
///
/// # Errors
///
/// Returns [`Error::Unsupported`] when the configured charset label is unknown.
pub fn resolve_encoding(charset: &CsvCharset) -> Result<&'static Encoding> {
    Encoding::for_label(charset.name().as_bytes())
        .ok_or_else(|| Error::Unsupported(format!("unsupported CSV charset: {}", charset.name())))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Wrap a byte reader with streaming BOM removal and UTF-8 transcoding.
///
/// The returned reader does not buffer the entire CSV document. This keeps the
/// facade's listener-based CSV path suitable for large inputs while locating
/// all charset and BOM policy in the format engine.
///
/// # Errors
///
/// Returns an error when the configured charset label is unknown.
pub fn decode_reader<R: Read>(
    reader: R,
    charset: &CsvCharset,
) -> Result<DecodeReaderBytes<R, Vec<u8>>> {
    let encoding = resolve_encoding(charset)?;
    Ok(DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding))
        .strip_bom(true)
        .build(reader))
}

include!("codec/csv_read_options.rs");

include!("codec/csv_write_options.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Detect a delimiter from a sample of text by counting candidate separators in
/// the first non-empty line.
#[must_use]
pub fn detect_delimiter(sample: &str) -> u8 {
    let first_line = sample.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
    let candidates = *b",;\t|";
    let mut best = b',';
    let mut best_count = 0usize;
    // Count outside of quotes.
    for &d in &candidates {
        let mut count = 0;
        let mut in_quotes = false;
        for b in first_line.bytes() {
            match b {
                b'"' => in_quotes = !in_quotes,
                x if x == d && !in_quotes => count += 1,
                _ => {}
            }
        }
        if count > best_count {
            best_count = count;
            best = d;
        }
    }
    best
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decode raw bytes to a UTF-8 `String`, stripping a UTF-8/UTF-16 BOM and
/// transcoding from the detected encoding if necessary.
#[must_use]
pub fn decode_bytes(bytes: &[u8]) -> String {
    // UTF-16 BOMs.
    if bytes.len() >= 2 {
        if bytes[0] == 0xFF && bytes[1] == 0xFE {
            let (cow, _, _) = encoding_rs::UTF_16LE.decode(&bytes[2..]);
            return cow.into_owned();
        }
        if bytes[0] == 0xFE && bytes[1] == 0xFF {
            let (cow, _, _) = encoding_rs::UTF_16BE.decode(&bytes[2..]);
            return cow.into_owned();
        }
    }
    // UTF-8 BOM.
    let body = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    };
    if let Ok(text) = std::str::from_utf8(body) {
        text.to_string()
    } else {
        // Fall back to Windows-1252, the most common non-UTF-8 CSV encoding.
        let (cow, _, _) = encoding_rs::WINDOWS_1252.decode(body);
        cow.into_owned()
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read CSV from any reader into a new single-sheet [`Workbook`].
///
/// # Errors
///
/// 读取输入、解析 CSV 或构建工作簿失败时返回错误。
pub fn read_csv<R: Read>(mut reader: R, opts: &CsvReadOptions) -> Result<Workbook> {
    let mut raw = Vec::new();
    reader.read_to_end(&mut raw)?;
    let text = decode_bytes(&raw);
    let delimiter = opts.delimiter.unwrap_or_else(|| detect_delimiter(&text));

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());

    let mut sheet = Sheet::new(&opts.sheet_name);
    let mut row_idx: u32 = 0;
    let mut record = csv::StringRecord::new();
    while rdr.read_record(&mut record).map_err(Error::from)? {
        for (col_idx, field) in record.iter().enumerate() {
            let cell = if opts.infer_types {
                infer_cell(field)
            } else if field.is_empty() {
                Cell::Empty
            } else {
                Cell::Text(field.to_string())
            };
            if !cell.is_empty() {
                let column = super::checked_column_index(col_idx)?;
                sheet.set(row_idx, column, cell);
            }
        }
        row_idx += 1;
    }

    let mut wb = Workbook::empty();
    wb.sheets.push(sheet);
    Ok(wb)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Infer a cell type from a raw CSV field: empty, boolean, number, ISO date, or
/// text. Conservative — anything ambiguous stays text.
#[must_use]
pub fn infer_cell(field: &str) -> Cell {
    if field.is_empty() {
        return Cell::Empty;
    }
    if field.eq_ignore_ascii_case("true") {
        return Cell::Bool(true);
    }
    if field.eq_ignore_ascii_case("false") {
        return Cell::Bool(false);
    }
    // Number: must parse fully and not have leading zeros that matter (keep
    // strings like "007" or phone numbers as text).
    if looks_numeric(field)
        && let Ok(n) = field.parse::<f64>()
    {
        return Cell::Number(n);
    }
    // ISO date YYYY-MM-DD → store as a serial Number (date styling is applied
    // separately; here we keep the literal text to avoid losing the format when
    // no style table is present).
    if NaiveDate::parse_from_str(field, "%Y-%m-%d").is_ok() {
        return Cell::Text(field.to_string());
    }
    Cell::Text(field.to_string())
}

fn looks_numeric(s: &str) -> bool {
    let t = s.trim();
    if t.is_empty() {
        return false;
    }
    // Reject values with leading zeros (likely identifiers) unless it's "0" or a
    // decimal like "0.5".
    let unsigned = t.strip_prefix(['-', '+']).unwrap_or(t);
    if unsigned.len() > 1 && unsigned.starts_with('0') && !unsigned.starts_with("0.") {
        return false;
    }
    t.parse::<f64>().is_ok()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Write a sheet of a workbook to CSV.
///
/// # Errors
///
/// 工作表索引不存在，或序列化、写入、刷新失败时返回错误。
pub fn write_csv<W: Write>(
    wb: &Workbook,
    sheet_idx: usize,
    writer: W,
    opts: &CsvWriteOptions,
) -> Result<()> {
    let sheet = wb
        .sheets
        .get(sheet_idx)
        .ok_or_else(|| Error::Other(format!("sheet index {sheet_idx} out of range")))?;
    let terminator = if opts.crlf {
        csv::Terminator::CRLF
    } else {
        csv::Terminator::Any(b'\n')
    };
    let mut w = csv::WriterBuilder::new()
        .delimiter(opts.delimiter)
        .terminator(terminator)
        // Rows may be ragged because trailing empty cells are trimmed for
        // compactness; allow varying field counts.
        .flexible(true)
        .from_writer(writer);

    let (max_row, max_col) = sheet.dimensions();
    for row in 0..max_row {
        let mut record: Vec<String> = Vec::with_capacity(max_col as usize);
        for col in 0..max_col {
            record.push(wb.display_cell(sheet_idx, row, col));
        }
        // Trim trailing empties for compactness.
        while record.last().is_some_and(String::is_empty) {
            record.pop();
        }
        w.write_record(&record).map_err(Error::from)?;
    }
    w.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_model::value::CellValue;

    #[test]
    fn detect() {
        assert_eq!(detect_delimiter("a,b,c"), b',');
        assert_eq!(detect_delimiter("a;b;c"), b';');
        assert_eq!(detect_delimiter("a\tb\tc"), b'\t');
        assert_eq!(detect_delimiter("\"a,b\";c"), b';');
    }

    #[test]
    fn bom_stripping() {
        let bytes = b"\xEF\xBB\xBFhello,world";
        assert_eq!(decode_bytes(bytes), "hello,world");
    }

    #[test]
    fn read_with_inference() {
        let data = "name,age,active\nAlice,30,true\nBob,25,false";
        let wb = read_csv(data.as_bytes(), &CsvReadOptions::default()).unwrap();
        let s = &wb.sheets[0];
        assert_eq!(s.value(0, 0), CellValue::Text("name".into()));
        assert_eq!(s.value(1, 1), CellValue::Number(30.0));
        assert_eq!(s.value(1, 2), CellValue::Bool(true));
        assert_eq!(s.value(2, 2), CellValue::Bool(false));
    }

    #[test]
    fn quoted_fields_with_commas() {
        let data = "a,\"b,c\",d";
        let wb = read_csv(data.as_bytes(), &CsvReadOptions::default()).unwrap();
        assert_eq!(wb.sheets[0].value(0, 1), CellValue::Text("b,c".into()));
    }

    #[test]
    fn leading_zero_kept_as_text() {
        let wb = read_csv("007,0.5,0".as_bytes(), &CsvReadOptions::default()).unwrap();
        assert_eq!(wb.sheets[0].value(0, 0), CellValue::Text("007".into()));
        assert_eq!(wb.sheets[0].value(0, 1), CellValue::Number(0.5));
        assert_eq!(wb.sheets[0].value(0, 2), CellValue::Number(0.0));
    }

    #[test]
    fn roundtrip() {
        let data = "x,y\n1,2\n3,4";
        let wb = read_csv(data.as_bytes(), &CsvReadOptions::default()).unwrap();
        let mut out = Vec::new();
        write_csv(&wb, 0, &mut out, &CsvWriteOptions::default()).unwrap();
        let s = String::from_utf8(out).unwrap();
        assert_eq!(s, "x,y\n1,2\n3,4\n");
    }

    #[test]
    fn write_ragged_rows() {
        // A row whose trailing cell is empty produces fewer fields than its
        // neighbours; the writer must allow that rather than erroring.
        let mut wb = Workbook::new();
        let s = wb.sheet_mut(0).unwrap();
        s.set(0, 0, Cell::Number(1.0));
        s.set(0, 1, Cell::Number(2.0));
        s.set(0, 2, Cell::Number(3.0));
        s.set(1, 0, Cell::Number(4.0)); // row 1 has only column A
        let mut out = Vec::new();
        write_csv(&wb, 0, &mut out, &CsvWriteOptions::default()).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "1,2,3\n4\n");
    }
}
