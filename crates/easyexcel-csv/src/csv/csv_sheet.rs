//! CSV 单工作表与有界行缓存模型。

use std::collections::VecDeque;

use easyexcel_io::{Error, Result};
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCellValue, CsvRow};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 单工作表、有序行的 CSV 模型。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvSheet<V: CsvCellValue = ModelCellValue> {
    name: String,
    row_cache_count: usize,
    last_row_index: Option<u32>,
    row_cache: VecDeque<CsvRow<V>>,
}

impl<V: CsvCellValue> CsvSheet<V> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用 Java 默认的一百行缓存创建工作表。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            row_cache_count: 100,
            last_row_index: None,
            row_cache: VecDeque::with_capacity(100),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回逻辑工作表名。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 设置有状态追加期望的首行位置。
    pub fn set_next_row_index(&mut self, next_row_index: u32) {
        self.last_row_index = next_row_index.checked_sub(1);
    }

    /// 返回最后创建的行号。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn last_row_index(&self) -> Option<u32> {
        self.last_row_index
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 查询仍处于缓存中的行。
    ///
    /// # Errors
    ///
    /// 行不存在或已经冲刷时返回不支持错误。
    pub fn row(&self, row_index: u32) -> Result<&CsvRow<V>> {
        self.row_cache
            .iter()
            .find(|row| row.row_index() == row_index)
            .ok_or_else(|| {
                Error::Unsupported("the CSV row does not exist or has been flushed".to_owned())
            })
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 移除并返回最近创建的行。
    pub fn take_last_row(&mut self) -> Option<CsvRow<V>> {
        self.row_cache.pop_back()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回超过缓存上限、可以冲刷的旧行。
    pub fn drain_flushable_rows(&mut self) -> Vec<CsvRow<V>> {
        let count = self.row_cache.len().saturating_sub(self.row_cache_count);
        self.row_cache.drain(..count).collect()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按严格递增顺序创建一行。
    ///
    /// # Errors
    ///
    /// 行号不是期望的下一行时返回 CSV 格式错误。
    pub fn try_create_row(&mut self, row_index: u32) -> Result<&mut CsvRow<V>> {
        let expected = self
            .last_row_index
            .map_or(0, |last_row_index| last_row_index.saturating_add(1));
        if row_index != expected {
            return Err(Error::Csv(format!(
                "CSV rows must be created in order: expected {expected}, got {row_index}"
            )));
        }
        self.last_row_index = Some(row_index);
        self.row_cache.push_back(CsvRow::new(row_index));
        self.row_cache
            .back_mut()
            .ok_or_else(|| Error::Csv("CSV row append produced no row".to_owned()))
    }
}
