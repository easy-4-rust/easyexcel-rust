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

    // ── resolve_encoding 测试 ──

    #[test]
    fn resolve_encoding_utf8() {
        let charset = CsvCharset::utf8();
        let enc = resolve_encoding(&charset).unwrap();
        assert_eq!(enc, encoding_rs::UTF_8);
    }

    #[test]
    fn resolve_encoding_unsupported() {
        let charset = CsvCharset::new("NOT-A-CHARSET");
        assert!(resolve_encoding(&charset).is_err());
    }

    // ── decode_reader 测试 ──

    #[test]
    fn decode_reader_passthrough_utf8() {
        let data = b"hello,world";
        let mut reader = decode_reader(&data[..], &CsvCharset::utf8()).unwrap();
        let mut out = String::new();
        reader.read_to_string(&mut out).unwrap();
        assert_eq!(out, "hello,world");
    }

    #[test]
    fn decode_reader_strips_bom() {
        let data = b"\xEF\xBB\xBFhello,world";
        let mut reader = decode_reader(&data[..], &CsvCharset::utf8()).unwrap();
        let mut out = String::new();
        reader.read_to_string(&mut out).unwrap();
        assert_eq!(out, "hello,world");
    }

    #[test]
    fn decode_reader_unsupported_charset() {
        let data = b"hello";
        assert!(decode_reader(&data[..], &CsvCharset::new("INVALID")).is_err());
    }

    // ── decode_bytes 更多路径 ──

    #[test]
    fn decode_bytes_utf16_le() {
        // "AB" in UTF-16LE: 0x41 0x00 0x42 0x00, with BOM 0xFF 0xFE
        let bytes = [0xFF, 0xFE, 0x41, 0x00, 0x42, 0x00];
        assert_eq!(decode_bytes(&bytes), "AB");
    }

    #[test]
    fn decode_bytes_utf16_be() {
        // "AB" in UTF-16BE: 0x00 0x41 0x00 0x42, with BOM 0xFE 0xFF
        let bytes = [0xFE, 0xFF, 0x00, 0x41, 0x00, 0x42];
        assert_eq!(decode_bytes(&bytes), "AB");
    }

    #[test]
    fn decode_bytes_non_utf8_fallback() {
        // Windows-1252: 0x80 is Euro sign (U+20AC)
        let bytes = [0x80];
        let result = decode_bytes(&bytes);
        assert_eq!(result, "\u{20AC}");
    }

    #[test]
    fn decode_bytes_no_bom() {
        let bytes = b"plain text";
        assert_eq!(decode_bytes(bytes), "plain text");
    }

    #[test]
    fn decode_bytes_empty() {
        assert_eq!(decode_bytes(b""), "");
    }

    // ── detect_delimiter 更多路径 ──

    #[test]
    fn detect_delimiter_pipe() {
        assert_eq!(detect_delimiter("a|b|c"), b'|');
    }

    #[test]
    fn detect_delimiter_defaults_to_comma_when_no_candidates() {
        assert_eq!(detect_delimiter("no delimiters here"), b',');
    }

    #[test]
    fn detect_delimiter_empty_input() {
        assert_eq!(detect_delimiter(""), b',');
    }

    #[test]
    fn detect_delimiter_skips_blank_lines() {
        assert_eq!(detect_delimiter("\n\na;b;c"), b';');
    }

    #[test]
    fn detect_delimiter_respects_quotes() {
        // 逗号在引号内不应计入；分号在引号外应该胜出
        assert_eq!(detect_delimiter("\"a,b\";c;d"), b';');
    }

    // ── infer_cell 测试 ──

    #[test]
    fn infer_cell_empty() {
        assert_eq!(infer_cell(""), Cell::Empty);
    }

    #[test]
    fn infer_cell_bool_true_case_insensitive() {
        assert_eq!(infer_cell("TRUE"), Cell::Bool(true));
        assert_eq!(infer_cell("True"), Cell::Bool(true));
    }

    #[test]
    fn infer_cell_bool_false_case_insensitive() {
        assert_eq!(infer_cell("FALSE"), Cell::Bool(false));
    }

    #[test]
    fn infer_cell_integer() {
        assert_eq!(infer_cell("42"), Cell::Number(42.0));
    }

    #[test]
    fn infer_cell_negative_number() {
        assert_eq!(infer_cell("-3.14"), Cell::Number(-3.14));
    }

    #[test]
    fn infer_cell_positive_sign() {
        assert_eq!(infer_cell("+7"), Cell::Number(7.0));
    }

    #[test]
    fn infer_cell_iso_date_stays_text() {
        // ISO 日期存为文本以避免丢失格式
        assert_eq!(infer_cell("2024-01-15"), Cell::Text("2024-01-15".to_string()));
    }

    #[test]
    fn infer_cell_leading_zero_stays_text() {
        assert_eq!(infer_cell("007"), Cell::Text("007".to_string()));
    }

    #[test]
    fn infer_cell_plain_text() {
        assert_eq!(infer_cell("hello"), Cell::Text("hello".to_string()));
    }

    // ── looks_numeric 内部函数测试 ──

    #[test]
    fn looks_numeric_various() {
        assert!(looks_numeric("1.5"));
        assert!(looks_numeric("-1"));
        assert!(looks_numeric("0.5"));
        assert!(looks_numeric("0"));
        assert!(!looks_numeric(""));
        assert!(!looks_numeric("  "));
        assert!(!looks_numeric("007"));
        assert!(!looks_numeric("abc"));
    }

    // ── read_csv 更多路径 ──

    #[test]
    fn read_csv_no_type_inference() {
        let data = "123,true,";
        let opts = CsvReadOptions {
            infer_types: false,
            ..Default::default()
        };
        let wb = read_csv(data.as_bytes(), &opts).unwrap();
        let s = &wb.sheets[0];
        assert_eq!(s.value(0, 0), CellValue::Text("123".into()));
        assert_eq!(s.value(0, 1), CellValue::Text("true".into()));
        // 空字段不会产生单元格
    }

    #[test]
    fn read_csv_explicit_delimiter() {
        let data = "a;b;c";
        let opts = CsvReadOptions {
            delimiter: Some(b';'),
            ..Default::default()
        };
        let wb = read_csv(data.as_bytes(), &opts).unwrap();
        assert_eq!(wb.sheets[0].value(0, 0), CellValue::Text("a".into()));
        assert_eq!(wb.sheets[0].value(0, 1), CellValue::Text("b".into()));
        assert_eq!(wb.sheets[0].value(0, 2), CellValue::Text("c".into()));
    }

    #[test]
    fn read_csv_custom_sheet_name() {
        let data = "x";
        let opts = CsvReadOptions {
            sheet_name: "MySheet".to_string(),
            ..Default::default()
        };
        let wb = read_csv(data.as_bytes(), &opts).unwrap();
        assert_eq!(wb.sheets[0].name, "MySheet");
    }

    #[test]
    fn read_csv_empty_input() {
        let wb = read_csv(b"" as &[u8], &CsvReadOptions::default()).unwrap();
        assert_eq!(wb.sheets[0].dimensions(), (0, 0));
    }

    #[test]
    fn read_csv_multiline() {
        let data = "a,b\nc,d\ne,f";
        let wb = read_csv(data.as_bytes(), &CsvReadOptions::default()).unwrap();
        let s = &wb.sheets[0];
        assert_eq!(s.value(0, 0), CellValue::Text("a".into()));
        assert_eq!(s.value(1, 0), CellValue::Text("c".into()));
        assert_eq!(s.value(2, 0), CellValue::Text("e".into()));
    }

    #[test]
    fn read_csv_with_bom_input() {
        let mut data = b"\xEF\xBB\xBF".to_vec();
        data.extend_from_slice(b"name,value\nfoo,123");
        let wb = read_csv(data.as_slice(), &CsvReadOptions::default()).unwrap();
        assert_eq!(wb.sheets[0].value(0, 0), CellValue::Text("name".into()));
        assert_eq!(wb.sheets[0].value(1, 1), CellValue::Number(123.0));
    }

    // ── write_csv 更多路径 ──

    #[test]
    fn write_csv_crlf() {
        let data = "a,b\n1,2";
        let wb = read_csv(data.as_bytes(), &CsvReadOptions::default()).unwrap();
        let opts = CsvWriteOptions {
            crlf: true,
            ..Default::default()
        };
        let mut out = Vec::new();
        write_csv(&wb, 0, &mut out, &opts).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "a,b\r\n1,2\r\n");
    }

    #[test]
    fn write_csv_tab_delimiter() {
        let mut wb = Workbook::new();
        let s = wb.sheet_mut(0).unwrap();
        s.set(0, 0, Cell::Text("x".to_string()));
        s.set(0, 1, Cell::Text("y".to_string()));
        let opts = CsvWriteOptions {
            delimiter: b'\t',
            ..Default::default()
        };
        let mut out = Vec::new();
        write_csv(&wb, 0, &mut out, &opts).unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "x\ty\n");
    }

    #[test]
    fn write_csv_invalid_sheet_index() {
        let wb = Workbook::empty();
        let mut out = Vec::new();
        let result = write_csv(&wb, 99, &mut out, &CsvWriteOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn write_csv_empty_sheet() {
        let wb = Workbook::new();
        // 新建工作簿有一个空工作表，写入后输出为空
        let mut out = Vec::new();
        write_csv(&wb, 0, &mut out, &CsvWriteOptions::default()).unwrap();
        assert_eq!(out.len(), 0);
    }

    // ── CsvReadOptions / CsvWriteOptions 默认值 ──

    #[test]
    fn csv_read_options_default() {
        let opts = CsvReadOptions::default();
        assert!(opts.delimiter.is_none());
        assert!(opts.infer_types);
        assert_eq!(opts.sheet_name, "Sheet1");
    }

    #[test]
    fn csv_write_options_default() {
        let opts = CsvWriteOptions::default();
        assert_eq!(opts.delimiter, b',');
        assert!(!opts.crlf);
    }
}
