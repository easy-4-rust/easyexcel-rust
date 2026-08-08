//! 对应 Java：`com.alibaba.excel.util.FileUtils`。
//!
//! 文件系统实现位于 `easyexcel-io`；这里仅保留 Java 兼容路径与错误类型适配。

#![allow(dead_code)]

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use easyexcel_io::io::file_utils::NamedTempFile;

use crate::core::excel_error::ExcelError;

/// Java `FileUtils.POI_FILES` 常量。
pub const POI_FILES: &str = "poifiles";
/// Java `FileUtils.EX_CACHE` 常量。
pub const EX_CACHE: &str = "excache";

/// 打开输入文件。对应 Java：`FileUtils#openInputStream`。
///
/// # Errors
///
/// 文件不存在、权限不足或底层文件系统拒绝打开时返回 I/O 错误。
pub fn open_input_stream(path: &Path) -> io::Result<std::fs::File> {
    easyexcel_io::io::file_utils::open_input_stream(path)
}

/// 写入完整字节内容。对应 Java：`FileUtils#writeToFile`。
///
/// # Errors
///
/// 目标无法创建或写入时返回文件 I/O 错误。
pub fn write_to_file(path: &Path, data: &[u8]) -> Result<(), ExcelError> {
    easyexcel_io::io::file_utils::write_to_file(path, data).map_err(ExcelError::from)
}

/// 对应 Java：`FileUtils.writeToFile(File, InputStream)`，覆盖目标文件。
///
/// # Errors
///
/// 输入读取、父目录创建或目标写入失败时返回错误。
pub fn write_reader_to_file<R: Read>(
    path: &Path,
    input: &mut R,
) -> Result<(), ExcelError> {
    write_reader_to_file_with_append(path, input, false)
}

/// 对应 Java：`FileUtils.writeToFile(File, InputStream, boolean)`。
///
/// # Errors
///
/// 输入读取、父目录创建或目标写入失败时返回错误。
pub fn write_reader_to_file_with_append<R: Read>(
    path: &Path,
    input: &mut R,
    append: bool,
) -> Result<(), ExcelError> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(append)
        .truncate(!append)
        .open(path)?;
    io::copy(input, &mut output)?;
    output.flush()?;
    Ok(())
}

/// 对应 Java：`FileUtils.readFileToByteArray(File)`。
///
/// # Errors
///
/// 文件不存在、不可读或读取失败时返回 I/O 错误。
pub fn read_file_to_byte_array(path: &Path) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}

/// 对应 Java：`FileUtils.createTmpFile(String)`。
///
/// `file_name` 作为临时文件名前缀，并在配置的临时目录下创建由 RAII 管理的文件。
///
/// # Errors
///
/// 临时目录或文件无法创建时返回 I/O 错误。
pub fn create_tmp_file(file_name: &str) -> io::Result<NamedTempFile> {
    let directory = get_temp_file_prefix();
    std::fs::create_dir_all(&directory)?;
    tempfile::Builder::new().prefix(file_name).tempfile_in(directory)
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 在配置的缓存目录中创建临时文件。
///
/// # Errors
///
/// 缓存目录无法创建或临时文件无法打开时返回 I/O 错误。
pub fn create_cache_tmp_file() -> io::Result<NamedTempFile> {
    easyexcel_io::io::file_utils::create_cache_tmp_file()
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 创建并返回 POI 兼容临时目录。
///
/// # Errors
///
/// 临时目录无法创建时返回 I/O 错误。
pub fn create_poi_files_directory() -> io::Result<PathBuf> {
    easyexcel_io::io::file_utils::create_poi_files_directory()
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 递归创建目录。
///
/// # Errors
///
/// 路径无效、权限不足或文件系统操作失败时返回错误。
pub fn create_directory(path: &Path) -> Result<(), ExcelError> {
    easyexcel_io::io::file_utils::create_directory(path).map_err(ExcelError::from)
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 删除文件或目录；不存在视为成功。
///
/// # Errors
///
/// 目标存在但无法删除时返回文件 I/O 错误。
pub fn delete(path: &Path) -> Result<(), ExcelError> {
    easyexcel_io::io::file_utils::delete(path).map_err(ExcelError::from)
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 返回临时文件根路径。
#[must_use]
pub fn get_temp_file_prefix() -> PathBuf {
    easyexcel_io::io::file_utils::get_temp_file_prefix()
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 设置临时文件根路径。
pub fn set_temp_file_prefix(prefix: impl Into<PathBuf>) {
    easyexcel_io::io::file_utils::set_temp_file_prefix(prefix);
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 返回 POI 兼容临时目录。
#[must_use]
pub fn get_poi_files_path() -> PathBuf {
    easyexcel_io::io::file_utils::get_poi_files_path()
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 设置 POI 兼容临时目录。
pub fn set_poi_files_path(path: impl Into<PathBuf>) {
    easyexcel_io::io::file_utils::set_poi_files_path(path);
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 返回缓存目录。
#[must_use]
pub fn get_cache_path() -> PathBuf {
    easyexcel_io::io::file_utils::get_cache_path()
}

/// 对应 Java：com.alibaba.excel.util.FileUtils。 设置缓存目录。
pub fn set_cache_path(path: impl Into<PathBuf>) {
    easyexcel_io::io::file_utils::set_cache_path(path);
}
