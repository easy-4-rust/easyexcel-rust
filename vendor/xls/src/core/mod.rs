//! The always-compiled core: data model, formula engine, and file I/O. No UI or
//! CLI dependencies are referenced here.

pub mod addr;
pub mod csv;
pub mod dates;
pub mod error;
pub mod formula;
pub mod model;
pub mod numfmt;
pub mod query;
pub mod stream;
pub mod styles;
pub mod value;
pub mod xls;
pub mod xlsx;

pub use addr::{CellAddress, CellRange};
pub use dates::DateSystem;
pub use error::{CellError, Error, Result};
pub use model::{Cell, Sheet, Workbook};
pub use value::CellValue;

use std::path::Path;

/// A detected spreadsheet file format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Xlsx,
    Xls,
    Csv,
}

impl Format {
    /// Guess the format from a path's extension.
    pub fn from_path(path: &Path) -> Option<Format> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("xlsx") | Some("xlsm") => Some(Format::Xlsx),
            Some("xls") => Some(Format::Xls),
            Some("csv") | Some("tsv") | Some("txt") => Some(Format::Csv),
            _ => None,
        }
    }

    /// Sniff the format from the leading magic bytes (falls back to CSV).
    pub fn from_magic(magic: &[u8]) -> Format {
        if xls::looks_like_cfb(magic) {
            Format::Xls
        } else if xlsx::looks_like_zip(magic) {
            Format::Xlsx
        } else {
            Format::Csv
        }
    }
}

/// Open a spreadsheet file, dispatching on its extension (then magic bytes).
pub fn open_path(path: &Path) -> Result<Workbook> {
    open_path_with_password(path, None)
}

/// Open a spreadsheet file, decrypting with `password` if it is a
/// password-protected (MS-OFFCRYPTO) XLSX. For unencrypted files the password
/// is ignored.
pub fn open_path_with_password(path: &Path, password: Option<&str>) -> Result<Workbook> {
    let fmt = match Format::from_path(path) {
        Some(f) => f,
        None => {
            // Sniff magic bytes.
            let mut buf = [0u8; 8];
            use std::io::Read;
            let mut f = std::fs::File::open(path)?;
            let n = f.read(&mut buf).unwrap_or(0);
            Format::from_magic(&buf[..n])
        }
    };
    // Note: "numbers stored as text" are coerced lazily by the formula engine
    // when a numeric function (SUM/AVERAGE/…) reads them, so the stored cells —
    // and on-disk fidelity — are preserved. Use `to-number` to convert them
    // permanently.
    match fmt {
        Format::Xlsx => xlsx::read_path_with_password(path, password),
        Format::Xls => xls::read_path(path),
        Format::Csv => {
            let f = std::fs::File::open(path)?;
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Sheet1")
                .to_string();
            csv::read_csv(
                f,
                &csv::CsvReadOptions {
                    sheet_name: name,
                    ..Default::default()
                },
            )
        }
    }
}

/// Save a workbook to a path, dispatching on its extension. For CSV the active
/// sheet (or sheet 0) is written.
pub fn save_path(wb: &Workbook, path: &Path) -> Result<()> {
    let fmt = Format::from_path(path)
        .ok_or_else(|| Error::Other(format!("cannot determine format for {}", path.display())))?;
    match fmt {
        Format::Xlsx => xlsx::write_path(wb, path),
        Format::Xls => xls::write_path(wb, path),
        Format::Csv => {
            let f = std::fs::File::create(path)?;
            csv::write_csv(
                wb,
                wb.active_sheet.min(wb.sheets.len().saturating_sub(1)),
                f,
                &csv::CsvWriteOptions::default(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn format_detection() {
        assert_eq!(Format::from_path(Path::new("a.xlsx")), Some(Format::Xlsx));
        assert_eq!(Format::from_path(Path::new("a.XLS")), Some(Format::Xls));
        assert_eq!(Format::from_path(Path::new("a.csv")), Some(Format::Csv));
        assert_eq!(Format::from_magic(b"PK\x03\x04xx"), Format::Xlsx);
        assert_eq!(Format::from_magic(&xls::CFB_MAGIC), Format::Xls);
    }
}
