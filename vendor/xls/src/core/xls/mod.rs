//! XLS (BIFF8, Excel 97–2003) reader and writer.
//!
//! NOTE: full implementation in progress. The public entry points below are the
//! frozen API the rest of the crate depends on.

use crate::core::error::Result;
use crate::core::model::Workbook;

mod biff;
mod reader;
mod sst;
mod writer;

pub use reader::read;
pub use writer::write;

/// Read an XLS workbook from a path.
pub fn read_path(path: &std::path::Path) -> Result<Workbook> {
    let file = std::fs::File::open(path)?;
    read(file)
}

/// Write a workbook to an XLS file at `path`.
pub fn write_path(wb: &Workbook, path: &std::path::Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    write(wb, file)
}

/// The OLE2/CFB magic header that prefixes every XLS file.
pub const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// Detect whether bytes look like an OLE2 compound file (and therefore XLS).
pub fn looks_like_cfb(magic: &[u8]) -> bool {
    magic.starts_with(&CFB_MAGIC)
}
