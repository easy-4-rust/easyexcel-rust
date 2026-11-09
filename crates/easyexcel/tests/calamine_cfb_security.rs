//! Regression fixtures from calamine's RUSTSEC-2026-0436 fix:
//! https://github.com/tafia/calamine/commit/b8ec4f09cba6aea7d8359ababc240c9ed064751b
//!
//! Both files are deliberately malformed OLE2/CFB inputs. They must only be
//! parsed by patched calamine (>= 0.38.0).

use std::io::Cursor;

use calamine::{Reader, Xls};

#[test]
fn malformed_cfb_short_directory_chunk_does_not_panic() {
    let bytes = include_bytes!("fixtures/calamine/cfb_dir_short_chunk_panic.xls");
    let workbook = Xls::new(Cursor::new(bytes.as_slice()))
        .expect("patched CFB parser should skip a short trailing directory chunk");
    assert_eq!(workbook.sheet_names(), vec!["Sheet1"]);
}

#[test]
fn malformed_cfb_cyclic_difat_is_rejected_without_unbounded_growth() {
    let bytes = include_bytes!("fixtures/calamine/cfb_difat_oom.xls");
    assert!(
        Xls::new(Cursor::new(bytes.as_slice())).is_err(),
        "patched CFB parser must reject a cyclic DIFAT chain"
    );
}
