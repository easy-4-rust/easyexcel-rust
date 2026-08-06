/// 对应 Java：无直接对应对象；Rust 架构扩展。 绑定逻辑工作表名称的中立 gzip 行 spill 读取器。
pub struct GzipCellSpillReader {
    sheet_name: String,
    inner: GzipCellRecordReader,
}

impl GzipCellSpillReader {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 打开已有 spill 文件，主要用于恢复或损坏诊断。
    ///
    /// # Errors
    ///
    /// 文件无法打开或 gzip 流无法初始化时返回错误。
    pub fn open_path(path: impl Into<PathBuf>, sheet_name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            sheet_name: sheet_name.into(),
            inner: GzipCellRecordReader::open_path(path)?,
        })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回完成后的 spill 状态。
    #[must_use]
    pub fn snapshot(&self) -> GzipCellSpillSnapshot {
        spill_snapshot(self.sheet_name.clone(), self.inner.snapshot())
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 读取下一行；到达 EOF 时返回 `None`。
    ///
    /// # Errors
    ///
    /// gzip 记录损坏或单元格行解码失败时返回错误。
    pub fn next_row(&mut self) -> Result<Option<Vec<GzipCellValue>>> {
        self.inner.next_row()
    }
}

