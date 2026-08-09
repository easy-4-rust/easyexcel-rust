//! Java `EasyExcelTempFileCreationStrategy` 兼容入口。

/// Java 同名策略的真实实现；文件生命周期与目录恢复由 `easyexcel-io` 承载。
pub use easyexcel_io::EasyExcelTempFileCreationStrategy;

/// 创建自动删除的临时文件。
pub use easyexcel_io::io::file_utils::create_temp_file;

/// 创建自动删除的临时目录。
pub use easyexcel_io::io::file_utils::create_temp_directory;
