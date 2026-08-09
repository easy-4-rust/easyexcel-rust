use std::path::Path;

use easyexcel_io::Result;
use easyexcel_model::model::Workbook;

use super::{read_with_password, write};

/// 从路径读取 XLSX 工作簿。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn read_path(path: &Path) -> Result<Workbook> { read_path_with_password(path, None) }

/// 从路径读取 XLSX，并按需解密 MS-OFFCRYPTO 文件。
///
/// # Errors
///
/// 文件无法打开、密码错误或 OOXML 内容无效时返回错误。
pub fn read_path_with_password(path: &Path, password: Option<&str>) -> Result<Workbook> {
    let file = std::fs::File::open(path)?;
    read_with_password(file, password)
}

/// 将工作簿写入 XLSX 路径。
///
/// # Errors
///
/// 文件无法创建或 OOXML 编码失败时返回错误。
pub fn write_path(workbook: &Workbook, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    write(workbook, file)
}

/// 判断文件头是否为 ZIP/OOXML 容器。
#[must_use]
pub fn looks_like_zip(magic: &[u8]) -> bool { easyexcel_io::looks_like_zip(magic) }
