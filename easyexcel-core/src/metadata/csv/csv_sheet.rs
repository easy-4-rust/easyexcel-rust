//! 对应 Java：`com.alibaba.excel.metadata.csv.CsvSheet`.

use std::collections::VecDeque;

use crate::excel_error::ExcelError;
use crate::util::work_book_util::RowCreator;

use super::csv_row::CsvRow;

/// Single-sheet, ordered-row CSV model.
#[derive(Debug, Clone, PartialEq)]
pub struct CsvSheet {
    name: String,
    row_cache_count: usize,
    last_row_index: Option<u32>,
    row_cache: VecDeque<CsvRow>,
}

impl CsvSheet {
    /// Creates an empty sheet with Java's default 100-row cache.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            row_cache_count: 100,
            last_row_index: None,
            row_cache: VecDeque::with_capacity(100),
        }
    }

    /// Returns the logical sheet name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the expected first row for a stateful append.
    pub fn set_next_row_index(&mut self, next_row_index: u32) {
        self.last_row_index = next_row_index.checked_sub(1);
    }

    /// Returns the last created row index.
    #[must_use]
    pub const fn last_row_index(&self) -> Option<u32> {
        self.last_row_index
    }

    /// Returns a cached row, or an error after it has been flushed.
    pub fn row(&self, row_index: u32) -> Result<&CsvRow, ExcelError> {
        self.row_cache
            .iter()
            .find(|row| row.row_index() == row_index)
            .ok_or_else(|| {
                ExcelError::Unsupported("the CSV row does not exist or has been flushed".to_owned())
            })
    }

    /// Removes and returns the most recently created row.
    pub fn take_last_row(&mut self) -> Option<CsvRow> {
        self.row_cache.pop_back()
    }

    /// Returns rows that exceed the configured cache size.
    pub fn drain_flushable_rows(&mut self) -> Vec<CsvRow> {
        let count = self.row_cache.len().saturating_sub(self.row_cache_count);
        self.row_cache.drain(..count).collect()
    }
}

impl RowCreator for CsvSheet {
    type Row<'a>
        = &'a mut CsvRow
    where
        Self: 'a;

    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>, ExcelError> {
        let expected = self
            .last_row_index
            .map_or(0, |last_row_index| last_row_index.saturating_add(1));
        if row_index != expected {
            return Err(ExcelError::Format(format!(
                "CSV rows must be created in order: expected {expected}, got {row_index}"
            )));
        }
        self.last_row_index = Some(row_index);
        self.row_cache.push_back(CsvRow::new(row_index));
        Ok(self.row_cache.back_mut().expect("just pushed"))
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn name_and_last_row_index_accessors() {
        // 对应 Java：CsvSheet 名称与最后行号
        let mut sheet = CsvSheet::new("Sheet1");
        assert_eq!(sheet.name(), "Sheet1");
        assert_eq!(sheet.last_row_index(), None);
        sheet.set_next_row_index(5);
        assert_eq!(sheet.last_row_index(), Some(4));
        sheet.set_next_row_index(0);
        assert_eq!(sheet.last_row_index(), None);
    }

    #[test]
    fn row_lookup_finds_cached_row_and_errors_otherwise() {
        // 对应 Java：缓存行查询，找不到返回错误
        let mut sheet = CsvSheet::new("Sheet1");
        sheet.create_row(0).expect("row 0 ok");
        sheet.create_row(1).expect("row 1 ok");
        assert_eq!(sheet.row(1).expect("found").row_index(), 1);
        let err = sheet.row(9).expect_err("missing row");
        assert!(err.to_string().contains("flushed"));
    }

    #[test]
    fn create_row_rejects_out_of_order_indexes() {
        // 对应 Java：CSV 行必须按顺序创建
        let mut sheet = CsvSheet::new("Sheet1");
        sheet.create_row(0).expect("row 0 ok");
        let err = sheet.create_row(2).expect_err("out of order");
        assert!(err.to_string().contains("must be created in order"));
    }

    #[test]
    fn drain_flushable_rows_exceeds_cache_count() {
        // 对应 Java：超过缓存行数后冲刷多余行
        let mut sheet = CsvSheet::new("Sheet1");
        for index in 0..102 {
            sheet.create_row(index).expect("row ok");
        }
        let drained = sheet.drain_flushable_rows();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].row_index(), 0);
        assert_eq!(drained[1].row_index(), 1);
        // 缓存内剩余行仍可查询
        assert_eq!(sheet.row(101).expect("cached").row_index(), 101);
        // 再冲刷则无多余行
        assert!(sheet.drain_flushable_rows().is_empty());
    }

    #[test]
    fn take_last_row_pops_most_recent() {
        // 对应 Java：弹出最近创建的行
        let mut sheet = CsvSheet::new("Sheet1");
        sheet.create_row(0).expect("row 0 ok");
        sheet.create_row(1).expect("row 1 ok");
        let popped = sheet.take_last_row().expect("popped");
        assert_eq!(popped.row_index(), 1);
        assert!(sheet.take_last_row().is_some());
        assert!(sheet.take_last_row().is_none());
    }
}
