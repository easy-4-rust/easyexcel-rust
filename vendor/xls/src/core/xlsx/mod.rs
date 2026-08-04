//! XLSX (OOXML SpreadsheetML) reader and writer.
//!
//! NOTE: full implementation in progress. The public entry points below are the
//! frozen API the rest of the crate depends on.

use crate::core::error::Result;
use crate::core::model::Workbook;

mod crypto;
mod reader;
mod shared_strings;
mod stream;
mod styles;
mod tables;
mod writer;
mod xmlutil;

pub use reader::{read, read_with_password};
pub use stream::stream;
pub use writer::write;

/// Read an XLSX workbook from a path.
pub fn read_path(path: &std::path::Path) -> Result<Workbook> {
    read_path_with_password(path, None)
}

/// Read an XLSX workbook from a path, decrypting with `password` if it is a
/// password-protected (MS-OFFCRYPTO) file.
pub fn read_path_with_password(path: &std::path::Path, password: Option<&str>) -> Result<Workbook> {
    let file = std::fs::File::open(path)?;
    read_with_password(file, password)
}

/// Write a workbook to an XLSX file at `path`.
pub fn write_path(wb: &Workbook, path: &std::path::Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    write(wb, file)
}

/// Detect whether bytes look like a ZIP (and therefore possibly XLSX).
pub fn looks_like_zip(magic: &[u8]) -> bool {
    magic.starts_with(b"PK\x03\x04") || magic.starts_with(b"PK\x05\x06")
}
