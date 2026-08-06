//! XLS (BIFF8, Excel 97–2003) reader and writer.
//!
//! NOTE: full implementation in progress. The public entry points below are the
//! frozen API the rest of the crate depends on.

use easyexcel_io::Result;
use easyexcel_model::model::Workbook;

mod biff;
mod reader;
mod sst;
mod writer;

pub use reader::read;
pub use writer::write;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Read an XLS workbook from a path.
///
/// # Errors
///
/// 文件无法打开，或 OLE2/BIFF8 内容无效时返回错误。
pub fn read_path(path: &std::path::Path) -> Result<Workbook> {
    let file = std::fs::File::open(path)?;
    read(file)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Write a workbook to an XLS file at `path`.
///
/// # Errors
///
/// 文件无法创建，或工作簿无法编码为 OLE2/BIFF8 时返回错误。
pub fn write_path(wb: &Workbook, path: &std::path::Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    write(wb, file)
}

/// The OLE2/CFB magic header that prefixes every XLS file.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Detect whether bytes look like an OLE2 compound file (and therefore XLS).
#[must_use]
pub fn looks_like_cfb(magic: &[u8]) -> bool {
    magic.starts_with(&CFB_MAGIC)
}
