/// 对应 Java：无直接对应对象；Rust 架构扩展。 中立单元格行 gzip spill 写入器。
pub struct GzipCellRecordWriter {
    inner: GzipRecordWriter,
}

impl GzipCellRecordWriter {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 在指定目录创建 spill 文件。
    ///
    /// # Errors
    ///
    /// 目录不可写或临时文件无法创建时返回错误。
    pub fn create(dir: &Path, prefix: &str, suffix: &str) -> Result<Self> {
        Ok(Self {
            inner: GzipRecordWriter::create(dir, prefix, suffix)?,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建自持有临时目录的 spill 文件。
    ///
    /// # Errors
    ///
    /// 临时目录或 spill 文件无法创建时返回错误。
    pub fn create_owned(prefix: &str, suffix: &str) -> Result<Self> {
        Ok(Self {
            inner: GzipRecordWriter::create_owned(prefix, suffix)?,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 写入一行中立单元格值。
    ///
    /// # Errors
    ///
    /// 行无法编码或压缩记录写入失败时返回错误。
    pub fn write_row(&mut self, cells: &[GzipCellValue]) -> Result<()> {
        self.inner.write_record(&encode_row(cells)?)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 刷新压缩输出。
    ///
    /// # Errors
    ///
    /// 压缩流刷新失败时返回错误。
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回 spill 状态。
    ///
    /// # Errors
    ///
    /// 压缩流刷新失败时返回错误。
    pub fn snapshot(&mut self) -> Result<GzipRecordSnapshot> {
        self.inner.snapshot()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 完成写入并切换到读取阶段。
    ///
    /// # Errors
    ///
    /// 压缩收尾或文件重新打开失败时返回错误。
    pub fn finish(self) -> Result<GzipCellRecordReader> {
        Ok(GzipCellRecordReader {
            inner: self.inner.finish()?,
        })
    }
}

