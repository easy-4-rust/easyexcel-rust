//! Streaming, case-insensitive OOXML package reader.

use std::collections::HashMap;
use std::io::{Read, Seek};

use easyexcel_io::{Error, Result};
use zip::ZipArchive;

use super::package::{RawRelationships, Relationships};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Owns ZIP access and hides ZIP implementation types from facade/event layers.
pub struct XlsxPackageReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    path_cache: HashMap<String, String>,
}

impl<R: Read + Seek> XlsxPackageReader<R> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Open a seekable OOXML ZIP stream.
    ///
    /// # Errors
    ///
    /// Returns a ZIP error when the stream is not a readable package.
    pub fn new(reader: R) -> Result<Self> {
        let archive = ZipArchive::new(reader).map_err(Error::from)?;
        let path_cache = super::package::path_cache(&archive);
        Ok(Self {
            archive,
            path_cache,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Return whether a part exists, using case-insensitive OPC lookup.
    #[must_use]
    pub fn contains(&self, path: &str) -> bool {
        self.path_cache.contains_key(&path.to_ascii_lowercase())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Open one package part as an opaque byte reader.
    ///
    /// # Errors
    ///
    /// Returns a ZIP error when the requested part does not exist or is corrupt.
    pub fn open_part<'a>(&'a mut self, path: &str) -> Result<Box<dyn Read + 'a>> {
        let actual = self
            .path_cache
            .get(&path.to_ascii_lowercase())
            .cloned()
            .unwrap_or_else(|| path.to_owned());
        let file = self.archive.by_name(&actual).map_err(Error::from)?;
        Ok(Box::new(file))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Return the uncompressed size of one package part.
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn part_size(&mut self, path: &str) -> Result<u64> {
        let actual = self
            .path_cache
            .get(&path.to_ascii_lowercase())
            .cloned()
            .unwrap_or_else(|| path.to_owned());
        let file = self.archive.by_name(&actual).map_err(Error::from)?;
        Ok(file.size())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Read internal relationships and omit external targets.
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn relationships(&mut self, path: &str) -> Result<Relationships> {
        super::package::read_relationships(&mut self.archive, &self.path_cache, path)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Read all relationships, retaining `TargetMode=External`.
    ///
    /// # Errors
    ///
    /// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
    pub fn raw_relationships(&mut self, path: &str) -> Result<RawRelationships> {
        super::package::read_raw_relationships(&mut self.archive, &self.path_cache, path)
    }
}
