/// 对应 Java：无直接对应对象；Rust 架构扩展。 绑定逻辑工作表名称的中立 gzip 行 spill 写入器。
pub struct GzipCellSpillWriter {
    sheet_name: String,
    inner: GzipCellRecordWriter,
}

impl GzipCellSpillWriter {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 在指定目录创建工作表 spill 文件。
    ///
    /// # Errors
    ///
    /// 目录不可写或临时文件无法创建时返回错误。
    pub fn create(
        dir: &Path,
        sheet_name: impl Into<String>,
        prefix: &str,
        suffix: &str,
    ) -> Result<Self> {
        Ok(Self {
            sheet_name: sheet_name.into(),
            inner: GzipCellRecordWriter::create(dir, prefix, suffix)?,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建自持有临时目录的工作表 spill 文件。
    ///
    /// # Errors
    ///
    /// 临时目录或 spill 文件无法创建时返回错误。
    pub fn create_owned(sheet_name: impl Into<String>, prefix: &str, suffix: &str) -> Result<Self> {
        Ok(Self {
            sheet_name: sheet_name.into(),
            inner: GzipCellRecordWriter::create_owned(prefix, suffix)?,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 写入一行中立单元格值。
    ///
    /// # Errors
    ///
    /// 行无法编码或压缩记录写入失败时返回错误。
    pub fn write_row(&mut self, cells: &[GzipCellValue]) -> Result<()> {
        self.inner.write_row(cells)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 刷新压缩输出。
    ///
    /// # Errors
    ///
    /// 压缩流刷新失败时返回错误。
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回当前 spill 状态。
    ///
    /// # Errors
    ///
    /// 压缩流刷新失败时返回错误。
    pub fn snapshot(&mut self) -> Result<GzipCellSpillSnapshot> {
        Ok(spill_snapshot(
            self.sheet_name.clone(),
            self.inner.snapshot()?,
        ))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 完成压缩并切换到读取阶段。
    ///
    /// # Errors
    ///
    /// 压缩收尾或文件重新打开失败时返回错误。
    pub fn finish(self) -> Result<GzipCellSpillReader> {
        Ok(GzipCellSpillReader {
            sheet_name: self.sheet_name,
            inner: self.inner.finish()?,
        })
    }
}

