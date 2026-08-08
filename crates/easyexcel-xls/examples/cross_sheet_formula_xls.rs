//! 生成包含 BIFF8 工作簿内部 3D 引用的 XLS。

use std::path::PathBuf;

use easyexcel_xls::biff8::{Biff8Book, Biff8Cell, Biff8Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("cross-sheet-formula.xls"), PathBuf::from);
    let mut book = Biff8Book::default();
    let source = book.create_sheet("Sheet1")?;
    source
        .cells
        .insert((0, 0), Biff8Cell::general(Biff8Value::Number(7.0)));
    source
        .cells
        .insert((1, 0), Biff8Cell::general(Biff8Value::Number(5.0)));
    let formulas = book.create_sheet("销售 数据")?;
    formulas.cells.insert(
        (0, 0),
        Biff8Cell::general(Biff8Value::Formula("Sheet1!A1*2".to_owned())),
    );
    formulas.cells.insert(
        (1, 0),
        Biff8Cell::general(Biff8Value::Formula("SUM(Sheet1!A1:A2)".to_owned())),
    );
    let quoted = book.create_sheet("结果")?;
    quoted.cells.insert(
        (0, 0),
        Biff8Cell::general(Biff8Value::Formula("'销售 数据'!$A$1+1".to_owned())),
    );
    quoted.cells.insert(
        (0, 1),
        Biff8Cell::general(Biff8Value::Formula("SUM('Sheet1:销售 数据'!A1)".to_owned())),
    );
    book.save_to_path(&path)?;
    Ok(())
}
