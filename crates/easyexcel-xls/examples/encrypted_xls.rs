//! 生成一个使用 BIFF8 `CryptoAPI` 密码加密的最小 XLS 文件。

use std::path::PathBuf;

use easyexcel_xls::biff8::{Biff8Book, Biff8Cell, Biff8Value};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("encrypted.xls"), PathBuf::from);
    let password = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "123456".to_owned());
    let mut book = Biff8Book::default();
    let sheet = book.create_sheet("Data")?;
    sheet.cells.insert(
        (0, 0),
        Biff8Cell::general(Biff8Value::Text("encrypted".to_owned())),
    );
    std::fs::write(path, book.to_cfb_bytes_with_password(Some(&password))?)?;
    Ok(())
}
