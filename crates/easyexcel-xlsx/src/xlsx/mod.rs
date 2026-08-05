//! XLSX (OOXML `SpreadsheetML`) reader and writer.
//!
//! NOTE: full implementation in progress. The public entry points below are the
//! frozen API the rest of the crate depends on.

use easyexcel_io::Result;
use easyexcel_model::model::Workbook;

mod crypto;
mod encrypt;
mod ooxml_package;
pub mod package;
mod reader;
mod shared_strings;
mod stream;
mod styles;
mod tables;
pub mod template_styles;
pub mod template_xml;
mod writer;
mod xmlutil;

pub use encrypt::{ReadWriteSeek, encrypt_package_to};
pub use ooxml_package::{OoxmlPackage, OoxmlZipEntry};
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

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_formula::Engine;
    use easyexcel_model::CellValue;
    use easyexcel_model::model::Cell;
    use std::io::Write;

    #[test]
    fn spill_results_are_persisted_into_written_xlsx() {
        // 动态数组公式 =SEQUENCE(2,2) 求值后产生 2x2 spill 区域。
        // 写出的 XLSX 必须把 anchor 之外的派生单元格以缓存值写出，
        // 读回时无需重算即可见全部结果（此前 spills 从不持久化）。
        let mut wb = Workbook::new();
        {
            let s = wb.sheet_mut(0).unwrap();
            s.set_a1(
                "A1",
                Cell::Formula {
                    expr: "=SEQUENCE(2,2)".to_owned(),
                    cached: CellValue::Empty,
                },
            );
        }
        Engine::new().recalc(&mut wb);
        let sheet = &wb.sheets[0];
        assert!(
            !sheet.spills.is_empty(),
            "recalc 必须为动态数组公式生成 spill"
        );

        let dir = std::env::temp_dir().join(format!("xls-spill-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("spill.xlsx");
        write_path(&wb, &path).expect("write xlsx");

        let back = read_path(&path).expect("read xlsx");
        let s = &back.sheets[0];
        // anchor A1 是公式；B1/A2/B2 是持久化的 spill 缓存值。
        assert!(matches!(s.get(0, 1), Some(Cell::Number(2.0))), "B1 = 2");
        assert!(matches!(s.get(1, 0), Some(Cell::Number(3.0))), "A2 = 3");
        assert!(matches!(s.get(1, 1), Some(Cell::Number(4.0))), "B2 = 4");
        assert!(
            matches!(s.get(0, 0), Some(Cell::Formula { .. })),
            "anchor 仍是公式单元格"
        );

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::io::stdout().flush();
    }
}
