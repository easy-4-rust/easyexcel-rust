/// Streaming gzip spill writer mirroring POI `GZIPSheetDataWriter`.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct GzipSheetDataWriter {
    inner: EngineSpillWriter,
    styles: Vec<JournalCellStyle>,
}

impl GzipSheetDataWriter {
    /// Creates a new gzip spill file under `dir` for `sheet_name`.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the tempfile cannot be created.
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn create(dir: &Path, sheet_name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            inner: EngineSpillWriter::create(dir, sheet_name, "easyexcel-sxssf-", ".xml.gz")
                .map_err(ExcelError::from)?,
            styles: Vec::new(),
        })
    }

    /// Creates a spill that owns its temporary directory (deleted on drop).
    ///
    /// # Errors
    ///
    /// Returns an I/O error when the temp directory or file cannot be created.
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn create_owned(sheet_name: impl Into<String>) -> Result<Self> {
        Ok(Self {
            inner: EngineSpillWriter::create_owned(sheet_name, "easyexcel-sxssf-", ".xml.gz")
                .map_err(ExcelError::from)?,
            styles: Vec::new(),
        })
    }

    /// Appends one data row (cell values) to the gzip spill.
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when encoding or writing fails.
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn write_row(&mut self, cells: &[CellValue]) -> Result<()> {
        let values = cells
            .iter()
            .map(to_spill_value)
            .collect::<Result<Vec<_>>>()?;
        self.inner.write_row(&values).map_err(ExcelError::from)
    }

    /// 写入包含最终 Handler 样式和行高的 Stateful journal 行。
    pub(crate) fn write_journal_row(&mut self, row: &JournalRow) -> Result<()> {
        let mut values = Vec::with_capacity(row.cells.len().saturating_add(1));
        for cell in &row.cells {
            let value = to_spill_value(&cell.value)?;
            if let Some(style) = &cell.style {
                let style_id = if let Some(index) = self.styles.iter().position(|item| item == style)
                {
                    u32::try_from(index).map_err(|_| {
                        ExcelError::Format("stateful journal style index exceeds u32".to_owned())
                    })?
                } else {
                    let index = self.styles.len();
                    self.styles.push(style.clone());
                    u32::try_from(index).map_err(|_| {
                        ExcelError::Format("stateful journal style count exceeds u32".to_owned())
                    })?
                };
                values.push(GzipCellValue::Styled {
                    value: Box::new(value),
                    style_id,
                });
            } else {
                values.push(value);
            }
        }
        values.push(GzipCellValue::JournalMetadata {
            row_height: row.row_height,
        });
        self.inner.write_row(&values).map_err(ExcelError::from)
    }

    /// Flushes buffered gzip bytes so magic / size are observable on disk.
    ///
    /// # Errors
    ///
    /// Returns an I/O error on flush failure.
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn flush(&mut self) -> Result<()> {
        self.inner.flush().map_err(ExcelError::from)
    }

    /// Returns a snapshot suitable for tests (gzip magic + sizes).
    ///
    /// # Errors
    ///
    /// Returns an I/O error when flushing or stating the file fails.
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn snapshot(&mut self) -> Result<GzipSpillSnapshot> {
        self.inner.snapshot().map_err(ExcelError::from)
    }

    /// Finishes the encoder and returns a readable spill handle.
    ///
    /// # Errors
    ///
    /// Returns an I/O error when finishing gzip or reopening the file fails.
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn finish(self) -> Result<GzipSpillReader> {
        Ok(GzipSpillReader {
            inner: self.inner.finish().map_err(ExcelError::from)?,
            styles: self.styles,
        })
    }
}
