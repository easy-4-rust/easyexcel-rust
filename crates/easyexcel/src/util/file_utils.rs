//! 对应 Java：`com.alibaba.excel.util.FileUtils`。
//!
//! 文件系统实现位于 `easyexcel-io`；这里仅保留 Java 兼容路径与错误类型适配。

#![allow(dead_code)]

use std::io;
use std::path::{Path, PathBuf};

use easyexcel_io::io::file_utils::NamedTempFile;

use crate::core::excel_error::ExcelError;

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
