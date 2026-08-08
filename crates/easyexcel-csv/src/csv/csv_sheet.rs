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

    /// 返回 Java `CsvSheet#getSheetName` 对应的逻辑名称。
    #[must_use]
    pub fn get_sheet_name(&self) -> &str {
        self.name()
    }

    /// 返回 Java `CsvSheet#getRowCacheCount` 对应的有界缓存行数。
    #[must_use]
    pub const fn row_cache_count(&self) -> usize {
        self.row_cache_count
    }
    pub const fn get_row_cache_count(&self) -> usize { self.row_cache_count() }

    /// 设置 Java `CsvSheet#setRowCacheCount` 对应的有界缓存行数。
    ///
    /// 零会被收敛为一，避免下一次追加在尚未交给输出器前丢弃当前行。
    pub fn set_row_cache_count(&mut self, row_cache_count: usize) {
        self.row_cache_count = row_cache_count.max(1);
        let excess = self.row_cache.len().saturating_sub(self.row_cache_count);
        self.row_cache.drain(..excess);
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
    pub const fn get_last_row_index(&self) -> Option<u32> { self.last_row_index() }

    /// 返回 Java `CsvSheet#getFirstRowNum` 的零基语义；空表返回 `None`。
    #[must_use]
    pub const fn first_row_num(&self) -> Option<u32> {
        if self.last_row_index.is_some() {
            Some(0)
        } else {
            None
        }
    }
    pub const fn get_first_row_num(&self) -> Option<u32> { self.first_row_num() }

    /// 返回 Java `CsvSheet#getLastRowNum` 的零基语义；空表返回 `None`。
    #[must_use]
    pub const fn last_row_num(&self) -> Option<u32> {
        self.last_row_index
    }
    pub const fn get_last_row_num(&self) -> Option<u32> { self.last_row_num() }

    /// 返回当前仍驻留在有界窗口内的物理行数。
    #[must_use]
    pub fn physical_number_of_rows(&self) -> usize {
        self.row_cache.len()
    }
    pub fn get_physical_number_of_rows(&self) -> usize { self.physical_number_of_rows() }

    /// 返回当前缓存窗口，语义对应 Java Lombok `getRowCache`。
    #[must_use]
    pub fn row_cache(&self) -> &VecDeque<CsvRow<V>> {
        &self.row_cache
    }
    pub fn get_row_cache(&self) -> &VecDeque<CsvRow<V>> { self.row_cache() }

    /// 返回当前缓存窗口的可变引用，避免调用方通过整体替换破坏行号单调性。
    pub fn row_cache_mut(&mut self) -> &mut VecDeque<CsvRow<V>> {
        &mut self.row_cache
    }

    /// 按行号返回缓存行，语义对应 Java `CsvSheet#getRow`。
    pub fn get_row(&self, row_index: u32) -> Result<&CsvRow<V>> {
        self.row(row_index)
    }

    /// 返回缓存行迭代器，语义对应 Java `rowIterator` / `iterator`。
    pub fn rows(&self) -> impl Iterator<Item = &CsvRow<V>> {
        self.row_cache.iter()
    }

    /// CSV 不支持随机删除行，对齐 Java `CsvSheet#removeRow` 的失败语义。
    pub fn remove_row(&mut self, _row_index: u32) -> Result<()> {
        Err(Error::Unsupported("csv cannot move row".to_owned()))
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

    /// 对齐 Java `CsvSheet#createRow` 的严格顺序创建入口。
    pub fn create_row(&mut self, row_index: u32) -> Result<&mut CsvRow<V>> {
        self.try_create_row(row_index)
    }

    /// CSV 的合并单元格在 Java 实现中是 no-op，并返回固定索引 `0`。
    #[must_use]
    pub const fn add_merged_region(&mut self) -> usize {
        0
    }

    /// CSV 不保存合并区域。
    #[must_use]
    pub const fn number_of_merged_regions(&self) -> usize {
        0
    }
    pub const fn get_num_merged_regions(&self) -> usize { self.number_of_merged_regions() }

    /// CSV 不保存列宽，Java getter 返回 `0`。
    #[must_use]
    pub const fn column_width(&self, _column_index: usize) -> usize {
        0
    }
    pub const fn get_column_width(&self, column_index: usize) -> usize {
        self.column_width(column_index)
    }

    /// CSV 列隐藏状态不持久化。
    #[must_use]
    pub const fn is_column_hidden(&self, _column_index: usize) -> bool {
        false
    }

    /// CSV 不保存冻结窗格；保留 Java no-op 调用体验。
    pub const fn create_freeze_pane(&mut self, _column_split: usize, _row_split: usize) {}

    /// CSV 不保存缩放；保留 Java no-op 调用体验。
    pub const fn set_zoom(&mut self, _scale: usize) {}

    /// CSV 本身不存储公式重算标志。
    #[must_use]
    pub const fn force_formula_recalculation(&self) -> bool {
        false
    }
    pub const fn get_force_formula_recalculation(&self) -> bool {
        self.force_formula_recalculation()
    }

    /// Java CSV Sheet 的不持久化视图属性。
    pub const fn get_default_column_width(&self) -> usize { 0 }
    pub const fn get_default_row_height(&self) -> u16 { 0 }
    pub const fn get_default_row_height_in_points(&self) -> f32 { 0.0 }
    pub const fn get_horizontally_center(&self) -> bool { false }
    pub const fn get_vertically_center(&self) -> bool { false }
    pub const fn is_display_zeros(&self) -> bool { false }
    pub const fn is_display_formulas(&self) -> bool { false }
    pub const fn is_print_gridlines(&self) -> bool { false }
    pub const fn is_selected(&self) -> bool { false }
    pub const fn is_right_to_left(&self) -> bool { false }
    pub const fn get_zoom(&self) -> usize { 0 }
    pub const fn get_top_row(&self) -> usize { 0 }
    pub const fn get_left_col(&self) -> usize { 0 }
    pub const fn get_margin(&self, _margin: usize) -> f64 { 0.0 }
    pub const fn set_default_column_width(&mut self, _width: usize) {}
    pub const fn set_default_row_height(&mut self, _height: u16) {}
    pub const fn set_default_row_height_in_points(&mut self, _height: f32) {}
    pub const fn set_horizontally_center(&mut self, _value: bool) {}
    pub const fn set_vertically_center(&mut self, _value: bool) {}
    pub const fn set_display_zeros(&mut self, _value: bool) {}
    pub const fn set_display_formulas(&mut self, _value: bool) {}
    pub const fn set_print_gridlines(&mut self, _value: bool) {}
    pub const fn set_selected(&mut self, _value: bool) {}
    pub const fn set_right_to_left(&mut self, _value: bool) {}
    pub const fn set_force_formula_recalculation(&mut self, _value: bool) {}
    pub const fn shift_rows(&mut self, _start: u32, _end: u32, _count: i32) {}
    pub const fn shift_columns(&mut self, _start: u16, _end: u16, _count: i32) {}
    pub fn row_iterator(&self) -> impl Iterator<Item = &CsvRow<V>> { self.rows() }
}
