//! OOXML ZIP package 的无损条目容器。
//!
//! 负责复制 ZIP 条目元数据、读取和重新打包，不解释工作簿、工作表或单元格语义。

use std::io::{Cursor, Read, Write};
use std::ops::{Deref, DerefMut};
use std::path::Path;

use easyexcel_io::Result;
use zip::CompressionMethod;
use zip::read::ZipArchive;
use zip::write::{SimpleFileOptions, ZipWriter};

/// 一个需要原样保留的 OOXML ZIP 条目。
#[derive(Debug, Clone)]
pub struct OoxmlZipEntry {
    /// ZIP 内路径。
    pub name: String,
    /// 是否为目录标记。
    pub is_dir: bool,
    /// 原压缩方式。
    pub compression: CompressionMethod,
    /// 可选 UNIX 权限位。
    pub unix_mode: Option<u32>,
    /// 条目原始内容。
    pub bytes: Vec<u8>,
}

/// 保持条目顺序与元数据的 OOXML ZIP 包。
#[derive(Debug, Clone, Default)]
pub struct OoxmlPackage {
    entries: Vec<OoxmlZipEntry>,
}

impl OoxmlPackage {
    /// 从 ZIP/OOXML 字节载入全部条目。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut archive = ZipArchive::new(Cursor::new(bytes.to_vec()))?;
        let mut entries = Vec::with_capacity(archive.len());
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            let mut content = Vec::new();
            if !entry.is_dir() {
                entry.read_to_end(&mut content)?;
            }
            entries.push(OoxmlZipEntry {
                name: entry.name().to_owned(),
                is_dir: entry.is_dir(),
                compression: entry.compression(),
                unix_mode: entry.unix_mode(),
                bytes: content,
            });
        }
        Ok(Self { entries })
    }

    /// 使用现有条目构建包。
    #[must_use]
    pub fn from_entries(entries: Vec<OoxmlZipEntry>) -> Self {
        Self { entries }
    }

    /// 重新打包为 XLSX ZIP 字节。
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let cursor = Cursor::new(Vec::new());
        let mut zip = ZipWriter::new(cursor);
        for entry in &self.entries {
            let mut options = SimpleFileOptions::default().compression_method(entry.compression);
            if let Some(mode) = entry.unix_mode {
                options = options.unix_permissions(mode);
            }
            if entry.is_dir {
                zip.add_directory(&entry.name, options)?;
            } else {
                zip.start_file(&entry.name, options)?;
                zip.write_all(&entry.bytes)?;
            }
        }
        Ok(zip.finish()?.into_inner())
    }

    /// 保存 ZIP 包到文件。
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        std::fs::write(path, self.to_bytes()?)?;
        Ok(())
    }

    /// 保存 ZIP 包到输出流。
    pub fn save_to_writer(&self, output: &mut dyn Write) -> Result<()> {
        output.write_all(&self.to_bytes()?)?;
        output.flush()?;
        Ok(())
    }
}

impl Deref for OoxmlPackage {
    type Target = Vec<OoxmlZipEntry>;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for OoxmlPackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl From<Vec<OoxmlZipEntry>> for OoxmlPackage {
    fn from(entries: Vec<OoxmlZipEntry>) -> Self {
        Self::from_entries(entries)
    }
}
