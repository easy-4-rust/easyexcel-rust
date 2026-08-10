/// Read side of a finished gzip spill (stream decode, constant memory).
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct GzipSpillReader {
    inner: EngineSpillReader,
    styles: Vec<JournalCellStyle>,
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
            .map(|row| {
                row.into_iter()
                    .filter_map(|value| match value {
                        GzipCellValue::JournalMetadata { .. }
                        | GzipCellValue::JournalMergeRange { .. } => None,
                        GzipCellValue::Styled { value, .. } => Some(from_spill_value(*value)),
                        value => Some(from_spill_value(value)),
                    })
                    .collect()
            })
            .transpose()
    }

    /// 解码 Stateful journal 行，并恢复去重样式及最终行高。
    pub(crate) fn next_journal_row(&mut self) -> Result<Option<JournalRow>> {
        let Some(values) = self.inner.next_row().map_err(ExcelError::from)? else {
            return Ok(None);
        };
        let mut row_height = None;
        let mut merge_ranges = Vec::new();
        let mut cells = Vec::with_capacity(values.len());
        for value in values {
            match value {
                GzipCellValue::JournalMetadata { row_height: height } => row_height = height,
                GzipCellValue::JournalMergeRange {
                    first_row,
                    last_row,
                    first_col,
                    last_col,
                } => merge_ranges.push(crate::write::merge_range::MergeRange {
                    first_row,
                    last_row,
                    first_column: first_col,
                    last_column: last_col,
                }),
                GzipCellValue::Styled { value, style_id } => {
                    let style = self
                        .styles
                        .get(usize::try_from(style_id).unwrap_or(usize::MAX))
                        .cloned()
                        .ok_or_else(|| {
                            ExcelError::Format(format!(
                                "stateful journal references missing style {style_id}"
                            ))
                        })?;
                    cells.push(JournalCell {
                        value: from_spill_value(*value)?,
                        style: Some(style),
                    });
                }
                value => cells.push(JournalCell::plain(from_spill_value(value)?)),
            }
        }
        Ok(Some(JournalRow {
            cells,
            row_height,
            merge_ranges,
        }))
    }
}
