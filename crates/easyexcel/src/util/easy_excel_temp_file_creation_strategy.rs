//! Java `EasyExcelTempFileCreationStrategy` 兼容入口。

/// 创建自动删除的临时文件。
pub use easyexcel_io::io::file_utils::create_temp_file;

/// 创建自动删除的临时目录。
pub use easyexcel_io::io::file_utils::create_temp_directory;
