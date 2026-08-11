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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_match_java_values() {
        assert_eq!(POI_FILES, "poifiles");
        assert_eq!(EX_CACHE, "excache");
    }

    #[test]
    fn write_to_file_and_read_back() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_write.bin");
        let data = b"hello rust";
        write_to_file(&path, data).unwrap();
        let read_back = read_file_to_byte_array(&path).unwrap();
        assert_eq!(read_back, data);
    }

    #[test]
    fn write_to_file_nonexistent_dir_returns_error() {
        let path = Path::new("/nonexistent_dir_xyz/test.txt");
        let result = write_to_file(path, b"data");
        assert!(result.is_err());
    }

    #[test]
    fn write_reader_to_file_writes_content() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("reader_out.txt");
        let content = b"from reader";
        let mut cursor = std::io::Cursor::new(content);
        write_reader_to_file(&path, &mut cursor).unwrap();
        let read_back = read_file_to_byte_array(&path).unwrap();
        assert_eq!(read_back, content);
    }

    #[test]
    fn write_reader_to_file_with_append_false_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("overwrite.txt");
        std::fs::write(&path, b"old").unwrap();
        let mut cursor = std::io::Cursor::new(b"new");
        write_reader_to_file_with_append(&path, &mut cursor, false).unwrap();
        let read_back = read_file_to_byte_array(&path).unwrap();
        assert_eq!(read_back, b"new");
    }

    #[test]
    fn write_reader_to_file_with_append_true_appends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("append.txt");
        std::fs::write(&path, b"hello").unwrap();
        let mut cursor = std::io::Cursor::new(b" world");
        write_reader_to_file_with_append(&path, &mut cursor, true).unwrap();
        let read_back = read_file_to_byte_array(&path).unwrap();
        assert_eq!(read_back, b"hello world");
    }

    #[test]
    fn read_file_to_byte_array_nonexistent_returns_error() {
        let result = read_file_to_byte_array(Path::new("/no_such_file_xyz.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn create_tmp_file_creates_valid_file() {
        let tmp = create_tmp_file("test_prefix").unwrap();
        let path = tmp.path().to_owned();
        assert!(path.exists());
        assert!(
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("test_prefix")
        );
    }

    #[test]
    fn create_cache_tmp_file_creates_valid_file() {
        let tmp = create_cache_tmp_file().unwrap();
        assert!(tmp.path().exists());
    }

    #[test]
    fn create_directory_creates_path() {
        let dir = tempfile::tempdir().unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        create_directory(&nested).unwrap();
        assert!(nested.exists());
    }

    #[test]
    fn delete_removes_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("to_delete.txt");
        std::fs::write(&path, b"delete me").unwrap();
        assert!(path.exists());
        delete(&path).unwrap();
        assert!(!path.exists());
    }

    #[test]
    fn delete_nonexistent_is_ok() {
        let path = Path::new("/no_such_file_to_delete_xyz.txt");
        // 不存在视为成功
        assert!(delete(path).is_ok());
    }

    #[test]
    fn get_and_set_temp_file_prefix() {
        let original = get_temp_file_prefix();
        let dir = tempfile::tempdir().unwrap();
        set_temp_file_prefix(dir.path());
        let updated = get_temp_file_prefix();
        assert_eq!(updated, dir.path());
        // 恢复原始值
        set_temp_file_prefix(original);
    }

    #[test]
    fn get_and_set_poi_files_path() {
        let original = get_poi_files_path();
        let dir = tempfile::tempdir().unwrap();
        set_poi_files_path(dir.path());
        let updated = get_poi_files_path();
        assert_eq!(updated, dir.path());
        set_poi_files_path(original);
    }

    #[test]
    fn get_and_set_cache_path() {
        let original = get_cache_path();
        let dir = tempfile::tempdir().unwrap();
        set_cache_path(dir.path());
        let updated = get_cache_path();
        assert_eq!(updated, dir.path());
        set_cache_path(original);
    }

    #[test]
    fn open_input_stream_opens_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("open_test.txt");
        std::fs::write(&path, b"data").unwrap();
        let mut file = open_input_stream(&path).unwrap();
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut file, &mut contents).unwrap();
        assert_eq!(contents, "data");
    }

    #[test]
    fn open_input_stream_nonexistent_returns_error() {
        let result = open_input_stream(Path::new("/no_such_file_open_xyz.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn create_poi_files_directory_returns_path() {
        let path = create_poi_files_directory().unwrap();
        assert!(path.exists());
    }
}
