//! 对应 Java： com.alibaba.excel.util.FileUtils.

#![allow(dead_code)]

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use tempfile::{Builder, NamedTempFile};

use crate::excel_error::ExcelError;

static DEFAULT_TEMP_FILE_PREFIX: OnceLock<PathBuf> = OnceLock::new();
static TEMP_FILE_PREFIX: OnceLock<RwLock<PathBuf>> = OnceLock::new();
static POI_FILES_PATH: OnceLock<RwLock<PathBuf>> = OnceLock::new();
static CACHE_PATH: OnceLock<RwLock<PathBuf>> = OnceLock::new();

fn default_temp_file_prefix() -> PathBuf {
    DEFAULT_TEMP_FILE_PREFIX
        .get_or_init(|| {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos());
            std::env::temp_dir().join(format!("easyexcel-{}-{nonce}", std::process::id()))
        })
        .clone()
}

fn temp_file_prefix_lock() -> &'static RwLock<PathBuf> {
    TEMP_FILE_PREFIX.get_or_init(|| RwLock::new(default_temp_file_prefix()))
}

fn poi_files_path_lock() -> &'static RwLock<PathBuf> {
    POI_FILES_PATH.get_or_init(|| RwLock::new(default_temp_file_prefix().join("poifiles")))
}

fn cache_path_lock() -> &'static RwLock<PathBuf> {
    CACHE_PATH.get_or_init(|| RwLock::new(default_temp_file_prefix().join("excache")))
}

fn read_configured_path(lock: &RwLock<PathBuf>) -> PathBuf {
    lock.read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

fn replace_configured_path(lock: &RwLock<PathBuf>, path: PathBuf) {
    *lock
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = path;
}

/// Mirrors `org.apache.commons.io.FileUtils#openInputStream`.
///
/// # Errors
///
/// 当 `path` 不存在或不可读时返回对应的 IO 错误。
pub fn open_input_stream(path: &Path) -> io::Result<std::fs::File> {
    fs::File::open(path)
}

/// Mirrors `com.alibaba.excel.util.FileUtils#writeToFile`.
///
/// # Errors
///
/// 当无法创建或写入文件时返回 [`ExcelError::Io`]。
pub fn write_to_file(path: &Path, data: &[u8]) -> Result<(), ExcelError> {
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

/// Mirrors `com.alibaba.excel.util.FileUtils#createCacheTmpFile`.
///
/// Creates the file under the currently configured cache directory.
///
/// # Errors
///
/// 当无法创建缓存目录或临时文件时返回对应的 IO 错误。
pub fn create_cache_tmp_file() -> io::Result<NamedTempFile> {
    let cache_path = get_cache_path();
    fs::create_dir_all(&cache_path)?;
    Builder::new()
        .prefix("easyexcel-cache-")
        .tempfile_in(cache_path)
}

/// Mirrors `com.alibaba.excel.util.FileUtils#createPoiFilesDirectory`.
///
/// The directory remains in place after this call, matching Java's process-wide
/// POI temp-file strategy instead of returning a short-lived `TempDir`.
///
/// # Errors
///
/// 当无法创建目录时返回对应的 IO 错误。
pub fn create_poi_files_directory() -> io::Result<PathBuf> {
    let path = get_poi_files_path();
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// Mirrors `org.apache.commons.io.FileUtils#forceMkdir` / `createDirectory`.
///
/// # Errors
///
/// 当目录无法创建时返回 [`ExcelError::Io`]。
pub fn create_directory(path: &Path) -> Result<(), ExcelError> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Mirrors `org.apache.commons.io.FileUtils#deleteQuietly` / `delete`.
///
/// # Errors
///
/// 当文件或目录无法删除时返回 [`ExcelError::Io`]（路径不存在视为成功）。
pub fn delete(path: &Path) -> Result<(), ExcelError> {
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

/// Mirrors `com.alibaba.excel.util.FileUtils#getTempFilePrefix`.
#[must_use]
pub fn get_temp_file_prefix() -> PathBuf {
    read_configured_path(temp_file_prefix_lock())
}

/// Mirrors `com.alibaba.excel.util.FileUtils#setTempFilePrefix`.
pub fn set_temp_file_prefix(prefix: impl Into<PathBuf>) {
    replace_configured_path(temp_file_prefix_lock(), prefix.into());
}

/// Mirrors `com.alibaba.excel.util.FileUtils#getPoiFilesPath`.
#[must_use]
pub fn get_poi_files_path() -> PathBuf {
    read_configured_path(poi_files_path_lock())
}

/// Mirrors `com.alibaba.excel.util.FileUtils#setPoiFilesPath`.
pub fn set_poi_files_path(path: impl Into<PathBuf>) {
    replace_configured_path(poi_files_path_lock(), path.into());
}

/// Mirrors `com.alibaba.excel.util.FileUtils#getCachePath`.
#[must_use]
pub fn get_cache_path() -> PathBuf {
    read_configured_path(cache_path_lock())
}

/// Mirrors `com.alibaba.excel.util.FileUtils#setCachePath`.
pub fn set_cache_path(path: impl Into<PathBuf>) {
    replace_configured_path(cache_path_lock(), path.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    static CONFIG_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn configured_paths_can_be_replaced_after_first_read() {
        let _guard = CONFIG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let old_prefix = get_temp_file_prefix();
        let old_poi = get_poi_files_path();
        let old_cache = get_cache_path();
        let directory = tempfile::tempdir().expect("temporary root");

        let prefix = directory.path().join("prefix");
        let poi = directory.path().join("poi");
        let cache = directory.path().join("cache");
        set_temp_file_prefix(&prefix);
        set_poi_files_path(&poi);
        set_cache_path(&cache);

        assert_eq!(get_temp_file_prefix(), prefix);
        assert_eq!(
            create_poi_files_directory().expect("create poi directory"),
            poi
        );
        assert!(poi.is_dir());
        let cache_file = create_cache_tmp_file().expect("create cache file");
        assert_eq!(cache_file.path().parent(), Some(cache.as_path()));

        set_temp_file_prefix(old_prefix);
        set_poi_files_path(old_poi);
        set_cache_path(old_cache);
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn open_input_stream_opens_existing_file() {
        // 对应 Java：FileUtils.openInputStream
        let directory = tempfile::tempdir().expect("temp root");
        let path = directory.path().join("in.txt");
        std::fs::write(&path, b"content").expect("write fixture");
        let file = open_input_stream(&path).expect("opens");
        assert!(file.metadata().expect("meta").len() > 0);

        let missing = directory.path().join("missing.txt");
        assert!(open_input_stream(&missing).is_err());
    }

    #[test]
    fn write_to_file_writes_bytes_and_reports_io_errors() {
        // 对应 Java：FileUtils.writeToFile
        let directory = tempfile::tempdir().expect("temp root");
        let path = directory.path().join("out.txt");
        write_to_file(&path, b"hello").expect("writes");
        assert_eq!(std::fs::read(&path).expect("read back"), b"hello");

        // 目标目录不存在时报 IO 错误
        let bad = directory.path().join("no-such-dir").join("out.txt");
        let err = write_to_file(&bad, b"x").expect_err("fails");
        assert!(matches!(err, ExcelError::Io(_)));
    }

    #[test]
    fn create_directory_creates_nested_dirs_and_errors_on_conflict() {
        // 对应 Java：FileUtils.createDirectory / forceMkdir
        let directory = tempfile::tempdir().expect("temp root");
        let nested = directory.path().join("a").join("b");
        create_directory(&nested).expect("created");
        assert!(nested.is_dir());

        // 以文件作为父路径时报错
        let blocker = directory.path().join("blocker");
        std::fs::write(&blocker, b"x").expect("fixture");
        let err = create_directory(&blocker.join("sub")).expect_err("fails");
        assert!(matches!(err, ExcelError::Io(_)));
    }

    #[test]
    fn delete_removes_files_and_directories_quietly() {
        // 对应 Java：FileUtils.delete / deleteQuietly
        let directory = tempfile::tempdir().expect("temp root");

        let file_path = directory.path().join("f.txt");
        std::fs::write(&file_path, b"x").expect("fixture");
        delete(&file_path).expect("file deleted");
        assert!(!file_path.exists());

        let dir_path = directory.path().join("d");
        std::fs::create_dir_all(&dir_path).expect("fixture");
        delete(&dir_path).expect("dir deleted");
        assert!(!dir_path.exists());

        // 不存在时静默成功
        delete(&directory.path().join("missing")).expect("no-op");
    }
}
