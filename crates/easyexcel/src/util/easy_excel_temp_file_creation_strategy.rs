//! Java `EasyExcelTempFileCreationStrategy` 兼容入口。

use std::path::PathBuf;

use tempfile::{NamedTempFile, TempDir};

/// 创建自动删除的临时文件。
pub fn create_temp_file() -> std::io::Result<NamedTempFile> {
    easyexcel_io::io::file_utils::create_temp_file()
}

/// 创建自动删除的临时目录。
pub fn create_temp_directory() -> std::io::Result<(TempDir, PathBuf)> {
    easyexcel_io::io::file_utils::create_temp_directory()
}
