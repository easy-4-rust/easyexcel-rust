//! Lightweight performance benchmarks (no external bench framework).
//!
//! Run with `cargo bench`. Measures, on a generated large sheet:
//!   * formula recalculation of a dependency chain,
//!   * XLSX write + read round-trip,
//!   * sparse-storage memory behavior (implicitly — only set cells are stored).

use std::io::Cursor;
use std::time::Instant;

use xls::core::Workbook;
use xls::core::formula::Engine;
use xls::core::model::Cell;
use xls::core::value::CellValue;

fn time<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let out = f();
    let elapsed = start.elapsed();
    println!("{label:<40} {:>10.3?}", elapsed);
    out
}

/// Build a workbook with `rows` rows: column A is a constant, column B is a
/// formula `=A{r}*2`, column C sums the running total `=C{r-1}+B{r}`.
fn build(rows: u32) -> Workbook {
    let mut wb = Workbook::new();
    let s = wb.sheet_mut(0).unwrap();
    for r in 0..rows {
        s.set(r, 0, Cell::Number((r + 1) as f64));
        s.set(
            r,
            1,
            Cell::Formula {
                expr: format!("=A{}*2", r + 1),
                cached: CellValue::Empty,
            },
        );
        let c = if r == 0 {
            "=B1".to_string()
        } else {
            format!("=C{}+B{}", r, r + 1)
        };
        s.set(
            r,
            2,
            Cell::Formula {
                expr: c,
                cached: CellValue::Empty,
            },
        );
    }
    wb
}

fn main() {
    println!("xls benchmarks\n{}", "-".repeat(52));

    for &rows in &[1_000u32, 10_000, 50_000] {
        println!("\n# {rows} rows ({} cells)", rows * 3);
        let mut wb = time("  build", || build(rows));

        let mut engine = Engine::new();
        let report = time("  recalc (full dependency graph)", || {
            engine.recalc(&mut wb)
        });
        assert_eq!(report.circular.len(), 0);

        let bytes = time("  xlsx write", || {
            let mut buf = Vec::new();
            xls::core::xlsx::write(&wb, Cursor::new(&mut buf)).unwrap();
            buf
        });
        println!("  {:<38} {:>9} KiB", "xlsx size", bytes.len() / 1024);

        let read_back = time("  xlsx read", || {
            xls::core::xlsx::read(Cursor::new(&bytes)).unwrap()
        });
        assert_eq!(read_back.sheets[0].cells.len(), wb.sheets[0].cells.len());
    }

    println!("\ndone.");
}
