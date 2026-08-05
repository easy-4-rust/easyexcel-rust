//! `rust_xlsxwriter` 生成后端与加密落盘边界。
//!
//! EasyExcel 门面通过本模块获得工作簿句柄；底层依赖、序列化、文件创建和
//! MS-OFFCRYPTO 包装由 `easyexcel-xlsx` 统一拥有。

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use easyexcel_io::{Error, Result};

pub use rust_xlsxwriter::{
    Color, Format, FormatAlign, FormatBorder, FormatPattern, FormatScript, FormatUnderline, Image,
    Note, ObjectMovement, Workbook, Worksheet,
};

use super::encrypt::{ReadWriteSeek, encrypt_package_to};

/// 创建 XLSX 生成工作簿。
#[must_use]
pub fn new_workbook() -> Workbook {
    Workbook::new()
}

/// 保存工作簿到文件，可选使用密码加密。
///
/// # Errors
///
/// XLSX 序列化、文件写入或加密失败时返回错误。
pub fn save_workbook(
    workbook: &mut Workbook,
    path: &Path,
    password: Option<&str>,
) -> Result<()> {
    let Some(password) = password else {
        return workbook.save(path).map_err(xlsxwriter_error);
    };
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    save_encrypted_workbook_to(workbook, password, &mut file)
}

/// 保存工作簿到任意输出流，可选使用密码加密。
///
/// # Errors
///
/// XLSX 序列化、流写入或加密失败时返回错误。
pub fn save_workbook_to_writer(
    workbook: &mut Workbook,
    output: &mut (dyn Write + Send),
    password: Option<&str>,
) -> Result<()> {
    if let Some(password) = password {
        let mut encrypted = std::io::Cursor::new(Vec::new());
        save_encrypted_workbook_to(workbook, password, &mut encrypted)?;
        output.write_all(encrypted.get_ref())?;
    } else {
        workbook
            .save_to_writer(&mut *output)
            .map_err(xlsxwriter_error)?;
    }
    output.flush()?;
    Ok(())
}

/// 序列化并加密工作簿到可读写 seek 流。
///
/// # Errors
///
/// XLSX 序列化或加密失败时返回错误。
pub fn save_encrypted_workbook_to(
    workbook: &mut Workbook,
    password: &str,
    output: &mut dyn ReadWriteSeek,
) -> Result<()> {
    let plaintext = workbook.save_to_buffer().map_err(xlsxwriter_error)?;
    encrypt_package_to(&plaintext, password, output)
}

fn xlsxwriter_error(error: impl std::fmt::Display) -> Error {
    Error::Xlsx(error.to_string())
}
