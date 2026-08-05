//! Streaming, case-insensitive OOXML package reader.

use std::collections::HashMap;
use std::io::{Read, Seek};

use easyexcel_io::{Error, Result};
use zip::ZipArchive;

use super::package::{RawRelationships, Relationships};

/// Owns ZIP access and hides ZIP implementation types from facade/event layers.
pub struct XlsxPackageReader<R: Read + Seek> {
    archive: ZipArchive<R>,
    path_cache: HashMap<String, String>,
}

impl<R: Read + Seek> XlsxPackageReader<R> {
    /// Open a seekable OOXML ZIP stream.
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

    /// Return whether a part exists, using case-insensitive OPC lookup.
    #[must_use]
    pub fn contains(&self, path: &str) -> bool {
        self.path_cache.contains_key(&path.to_ascii_lowercase())
    }

    /// Open one package part as an opaque byte reader.
    ///
    /// # Errors
    ///
    /// Returns a ZIP error when the requested part does not exist or is corrupt.
    pub fn open_part<'a>(&'a mut self, path: &str) -> Result<Box<dyn Read + 'a>> {
        let actual = self
            .path_cache
            .get(&path.to_ascii_lowercase())
            .map_or(path, String::as_str);
        let file = self.archive.by_name(actual).map_err(Error::from)?;
        Ok(Box::new(file))
    }

    /// Return the uncompressed size of one package part.
    pub fn part_size(&mut self, path: &str) -> Result<u64> {
        let actual = self
            .path_cache
            .get(&path.to_ascii_lowercase())
            .map_or(path, String::as_str);
        let file = self.archive.by_name(actual).map_err(Error::from)?;
        Ok(file.size())
    }

    /// Read internal relationships and omit external targets.
    pub fn relationships(&mut self, path: &str) -> Result<Relationships> {
        super::package::read_relationships(&mut self.archive, &self.path_cache, path)
    }

    /// Read all relationships, retaining `TargetMode=External`.
    pub fn raw_relationships(&mut self, path: &str) -> Result<RawRelationships> {
        super::package::read_raw_relationships(&mut self.archive, &self.path_cache, path)
    }
}
