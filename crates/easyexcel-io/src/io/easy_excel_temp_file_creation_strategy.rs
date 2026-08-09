use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use tempfile::{Builder, NamedTempFile, TempDir};

/// 可在临时目录被外部清理后自动恢复的文件创建策略。
///
/// 对应 Java：`com.alibaba.excel.util.EasyExcelTempFileCreationStrategy`。
/// Java 依靠 `deleteOnExit` 管理生命周期；Rust 返回 RAII 所有权守卫，调用方
/// 持有守卫期间路径有效，守卫释放后自动清理。
#[derive(Debug, Default)]
pub struct EasyExcelTempFileCreationStrategy {
    directory: RwLock<Option<PathBuf>>,
}

impl EasyExcelTempFileCreationStrategy {
    /// Java 默认构造使用系统临时目录下的 `poifiles` 子目录。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            directory: RwLock::new(None),
        }
    }

    /// 使用调用方提供的现有目录创建策略。
    ///
    /// `directory` 若在后续被删除，与 Java 一致回退到系统临时目录下重新创建
    /// `poifiles`，而不是假定旧路径仍然可用。
    #[must_use]
    pub fn with_directory(directory: impl Into<PathBuf>) -> Self {
        Self::from_directory(Some(directory.into()))
    }

    /// 按 Java 可空 `File` 构造参数创建策略。
    ///
    /// `None` 表示使用系统临时目录，`Some` 表示优先使用指定的现有目录。
    #[must_use]
    pub const fn from_directory(directory: Option<PathBuf>) -> Self {
        Self {
            directory: RwLock::new(directory),
        }
    }

    /// 创建带指定前缀和后缀的临时文件。
    ///
    /// # 参数
    ///
    /// - `prefix`：文件名前缀。
    /// - `suffix`：文件名后缀。
    ///
    /// # 返回
    ///
    /// 返回持有文件生命周期的 [`NamedTempFile`]。
    ///
    /// # Errors
    ///
    /// 目标目录或临时文件无法创建时返回 I/O 错误。
    pub fn create_temp_file(&self, prefix: &str, suffix: &str) -> io::Result<NamedTempFile> {
        let directory = self.ensure_directory()?;
        Builder::new()
            .prefix(prefix)
            .suffix(suffix)
            .tempfile_in(directory)
    }

    /// 创建带指定前缀的临时目录。
    ///
    /// # 参数
    ///
    /// - `prefix`：目录名前缀。
    ///
    /// # 返回
    ///
    /// 返回持有目录生命周期的 [`TempDir`]。
    ///
    /// # Errors
    ///
    /// 父目录或临时目录无法创建时返回 I/O 错误。
    pub fn create_temp_directory(&self, prefix: &str) -> io::Result<TempDir> {
        let directory = self.ensure_directory()?;
        Builder::new().prefix(prefix).tempdir_in(directory)
    }

    /// 返回当前有效的临时文件父目录；目录不存在时立即恢复。
    ///
    /// # Errors
    ///
    /// 系统临时目录下的 `poifiles` 无法创建时返回 I/O 错误。
    pub fn directory(&self) -> io::Result<PathBuf> {
        self.ensure_directory()
    }

    fn ensure_directory(&self) -> io::Result<PathBuf> {
        if let Some(directory) = self.current_existing_directory() {
            return Ok(directory);
        }

        let mut directory = self
            .directory
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(current) = directory.as_ref().filter(|path| path.is_dir()) {
            return Ok(current.clone());
        }

        let fallback = std::env::temp_dir().join(Self::POIFILES);
        fs::create_dir_all(&fallback)?;
        *directory = Some(fallback.clone());
        Ok(fallback)
    }

    fn current_existing_directory(&self) -> Option<PathBuf> {
        self.directory
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_deref()
            .filter(|path| path.is_dir())
            .map(Path::to_path_buf)
    }

    /// Java POI 临时文件子目录名称。
    pub const POIFILES: &'static str = "poifiles";

    /// Java 控制 JVM 退出时删除临时文件的系统属性名。
    ///
    /// Rust 通过返回的 RAII 守卫直接管理生命周期，不读取该属性。
    pub const DELETE_FILES_ON_EXIT: &'static str = "poi.delete.tmp.files.on.exit";
}
