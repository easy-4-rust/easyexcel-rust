//! 对应 Java： com.alibaba.excel.util.EasyExcelTempFileCreationStrategy.
//!
//! Java swaps POI's `TempFileCreationStrategy` for one that honours the
//! EasyExcel cache path. The Rust port uses `tempfile::TempDir` /
//! `NamedTempFile` directly, so these helpers preserve the 1:1 Java
//! file mapping while delegating to `tempfile` under the hood.

#![allow(dead_code)]

use std::path::PathBuf;

use tempfile::{NamedTempFile, TempDir};

/// Mirrors `com.alibaba.excel.util.EasyExcelTempFileCreationStrategy#createTempFile`.
pub fn create_temp_file() -> std::io::Result<NamedTempFile> {
    NamedTempFile::new()
}

/// Mirrors `com.alibaba.excel.util.EasyExcelTempFileCreationStrategy#createTempDirectory`.
pub fn create_temp_directory() -> std::io::Result<(TempDir, PathBuf)> {
    let dir = TempDir::new()?;
    let path = dir.path().to_path_buf();
    Ok((dir, path))
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn create_temp_file_makes_named_file() {
        // 对应 Java：EasyExcelTempFileCreationStrategy.createTempFile
        let file = create_temp_file().expect("temp file");
        assert!(file.path().exists());
    }

    #[test]
    fn create_temp_directory_makes_directory() {
        // 对应 Java：EasyExcelTempFileCreationStrategy.createTempDirectory
        let (dir, path) = create_temp_directory().expect("temp dir");
        assert!(path.is_dir());
        assert_eq!(dir.path(), path.as_path());
    }
}
