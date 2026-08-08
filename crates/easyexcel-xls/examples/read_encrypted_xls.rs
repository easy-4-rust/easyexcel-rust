//! 使用调用级密码读取 BIFF8 `CryptoAPI` 加密 XLS。

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: read_encrypted_xls <path> <password>")?;
    let password = std::env::args()
        .nth(2)
        .ok_or("usage: read_encrypted_xls <path> <password>")?;
    let workbook = easyexcel_xls::read_path_with_password(&path, Some(&password))?;
    let sheet = workbook.sheets.first().ok_or("workbook has no sheets")?;
    println!("{}!A1={:?}", sheet.name, sheet.value(0, 0));
    Ok(())
}
