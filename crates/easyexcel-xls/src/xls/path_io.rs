use std::path::Path;

use easyexcel_io::Result;
use easyexcel_model::model::Workbook;

use super::{read_with_password, write_with_password};

/// OLE2/CFB 文件头。
pub const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];

/// 从路径读取 XLS 工作簿。
///
/// # Errors
///
/// 文件无法打开，或 OLE2/BIFF8 内容无效时返回错误。
pub fn read_path(path: &Path) -> Result<Workbook> {
    read_path_with_password(path, None)
}

/// 从路径读取 XLS，并在存在 `FILEPASS` 时使用密码解密。
///
/// # Errors
///
/// 文件无法打开、内容无效、未提供密码或密码错误时返回错误。
pub fn read_path_with_password(path: &Path, password: Option<&str>) -> Result<Workbook> {
    let file = std::fs::File::open(path)?;
    read_with_password(file, password)
}

/// 将工作簿写入 XLS 路径。
///
/// # Errors
///
/// 文件无法创建，或工作簿无法编码为 OLE2/BIFF8 时返回错误。
pub fn write_path(workbook: &Workbook, path: &Path) -> Result<()> {
    write_path_with_password(workbook, path, None)
}

/// 使用调用级密码将工作簿写入 XLS 路径。
///
/// # Errors
///
/// 文件无法创建，模型无法无损转换，或 CryptoAPI/BIFF8/OLE2 写出失败时返回错误。
pub fn write_path_with_password(
    workbook: &Workbook,
    path: &Path,
    password: Option<&str>,
) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(path)?;
    write_with_password(workbook, file, password)
}

/// 判断文件头是否为 OLE2/CFB。
#[must_use]
pub fn looks_like_cfb(magic: &[u8]) -> bool {
    easyexcel_io::looks_like_cfb(magic)
}
