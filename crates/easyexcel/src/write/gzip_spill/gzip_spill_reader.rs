/// Read side of a finished gzip spill (stream decode, constant memory).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct GzipSpillReader {
    inner: EngineSpillReader,
}

impl GzipSpillReader {
    /// Returns spill metadata after finish.
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn snapshot(&self) -> GzipSpillSnapshot {
        self.inner.snapshot()
    }

    /// Decodes the next spilled row, or `None` at EOF.
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when the stream is corrupt.
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn next_row(&mut self) -> Result<Option<Vec<CellValue>>> {
        self.inner
            .next_row()
            .map_err(ExcelError::from)?
            .map(|row| row.into_iter().map(from_spill_value).collect())
            .transpose()
    }
}

