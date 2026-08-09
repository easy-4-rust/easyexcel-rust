//! CSV 单工作表与有界行缓存模型。

use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};

use easyexcel_io::{Error, Result};
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCellValue, CsvRow};

static NEXT_CSV_SHEET_ID: AtomicUsize = AtomicUsize::new(1);

/// 对应 Java：com.alibaba.excel.metadata.csv.CsvSheet。 单工作表、有序行的 CSV 模型。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvSheet<V: CsvCellValue = ModelCellValue> {
    identity: usize,
    name: String,
    csv_workbook_id: Option<usize>,
    out: String,
    csv_printer_initialized: bool,
    row_cache_count: usize,
    last_row_index: Option<u32>,
    row_cache: VecDeque<CsvRow<V>>,
}

impl<V: CsvCellValue> CsvSheet<V> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用 Java 默认的一百行缓存创建工作表。
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            identity: NEXT_CSV_SHEET_ID.fetch_add(1, Ordering::Relaxed),
            name: name.into(),
            csv_workbook_id: None,
            out: String::new(),
            csv_printer_initialized: false,
            row_cache_count: 100,
            last_row_index: None,
            row_cache: VecDeque::with_capacity(100),
        }
    }

    /// 返回工作表稳定身份，供 Row/Cell 替代 Java 父对象引用。
    #[must_use]
    pub const fn identity(&self) -> usize {
        self.identity
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

    /// 返回父工作簿稳定身份。对应 Java Lombok `getCsvWorkbook`。
    #[must_use] pub const fn get_csv_workbook(&self) -> Option<usize> { self.csv_workbook_id }
    /// 设置父工作簿稳定身份，避免 Rust 自引用结构。
    pub fn set_csv_workbook(&mut self, value: Option<usize>) {
        self.csv_workbook_id = value;
        for row in &mut self.row_cache {
            row.set_csv_workbook(value);
            row.set_csv_sheet(Some(self.identity));
        }
    }
    /// 返回输出缓冲。对应 Java Lombok `getOut`。
    #[must_use] pub fn get_out(&self) -> &str { &self.out }
    /// 设置输出缓冲。对应 Java Lombok `setOut`。
    pub fn set_out(&mut self, value: impl Into<String>) { self.out = value.into(); }
    /// 返回 CSV printer 初始化状态的后端中立映射。
    #[must_use] pub const fn get_csv_printer(&self) -> bool { self.csv_printer_initialized }
    /// 设置 CSV printer 初始化状态的后端中立映射。
    pub const fn set_csv_printer(&mut self, value: bool) { self.csv_printer_initialized = value; }

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
    /// 替换缓存窗口。对应 Java Lombok `setRowCache`。
    pub fn set_row_cache(&mut self, value: VecDeque<CsvRow<V>>) {
        self.row_cache = value;
        for row in &mut self.row_cache {
            row.set_csv_workbook(self.csv_workbook_id);
            row.set_csv_sheet(Some(self.identity));
        }
        self.last_row_index = self.row_cache.back().map(CsvRow::row_index);
    }

    /// 在缓存达到阈值时返回可冲刷行。对应 Java `printData()`。
    #[must_use]
    pub fn print_data(&mut self) -> Vec<CsvRow<V>> {
        if self.row_cache.len() >= self.row_cache_count {
            self.drain_flushable_rows()
        } else {
            Vec::new()
        }
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
        let mut row = CsvRow::new(row_index);
        row.set_csv_workbook(self.csv_workbook_id);
        row.set_csv_sheet(Some(self.identity));
        self.row_cache.push_back(row);
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
    pub const fn add_merged_region_unsafe(&mut self) -> usize { 0 }
    #[must_use] pub const fn get_merged_region(&self, _index: usize) -> Option<&str> { None }
    #[must_use] pub const fn get_merged_regions(&self) -> Vec<&str> { Vec::new() }
    pub const fn remove_merged_region(&mut self, _index: usize) {}
    pub const fn remove_merged_regions(&mut self, _indexes: &[usize]) {}
    pub const fn validate_merged_regions(&self) {}

    /// CSV 不保存列样式、轮廓、分页符或窗格。
    #[must_use] pub const fn get_column_style(&self, _column: usize) -> Option<&str> { None }
    #[must_use] pub const fn get_column_width_in_pixels(&self, _column: usize) -> f32 { 0.0 }
    #[must_use] pub const fn get_column_outline_level(&self, _column: usize) -> u8 { 0 }
    #[must_use] pub const fn get_column_breaks(&self) -> Vec<usize> { Vec::new() }
    #[must_use] pub const fn get_row_breaks(&self) -> Vec<usize> { Vec::new() }
    #[must_use] pub const fn is_column_broken(&self, _column: usize) -> bool { false }
    #[must_use] pub const fn is_row_broken(&self, _row: usize) -> bool { false }
    #[must_use] pub const fn get_pane_information(&self) -> Option<&str> { None }
    pub const fn set_column_break(&mut self, _column: usize) {}
    pub const fn remove_column_break(&mut self, _column: usize) {}
    pub const fn set_row_break(&mut self, _row: usize) {}
    pub const fn remove_row_break(&mut self, _row: usize) {}
    pub const fn group_column(&mut self, _from: usize, _to: usize) {}
    pub const fn ungroup_column(&mut self, _from: usize, _to: usize) {}
    pub const fn group_row(&mut self, _from: usize, _to: usize) {}
    pub const fn ungroup_row(&mut self, _from: usize, _to: usize) {}
    pub const fn set_column_group_collapsed(&mut self, _column: usize, _collapsed: bool) {}
    pub const fn set_row_group_collapsed(&mut self, _row: usize, _collapsed: bool) {}
    pub const fn set_column_hidden(&mut self, _column: usize, _hidden: bool) {}
    pub const fn set_default_column_style(&mut self, _column: usize, _style: Option<&str>) {}
    pub const fn auto_size_column(&mut self, _column: usize, _use_merged_cells: bool) {}
    pub const fn create_split_pane(&mut self, _x_split: usize, _y_split: usize, _left: usize, _top: usize) {}
    pub const fn show_in_pane(&mut self, _top_row: usize, _left_column: usize) {}

    /// Java CSV Sheet 的打印/显示 no-op 状态。
    #[must_use] pub const fn is_display_gridlines(&self) -> bool { false }
    #[must_use] pub const fn is_display_row_col_headings(&self) -> bool { false }
    #[must_use] pub const fn is_print_row_and_column_headings(&self) -> bool { false }
    #[must_use] pub const fn get_autobreaks(&self) -> bool { false }
    #[must_use] pub const fn get_display_guts(&self) -> bool { false }
    #[must_use] pub const fn get_fit_to_page(&self) -> bool { false }
    #[must_use] pub const fn get_row_sums_below(&self) -> bool { false }
    #[must_use] pub const fn get_row_sums_right(&self) -> bool { false }
    #[must_use] pub const fn get_scenario_protect(&self) -> bool { false }
    #[must_use] pub const fn get_protect(&self) -> bool { false }
    pub const fn set_display_gridlines(&mut self, _value: bool) {}
    pub const fn set_display_row_col_headings(&mut self, _value: bool) {}
    pub const fn set_print_row_and_column_headings(&mut self, _value: bool) {}
    pub const fn set_autobreaks(&mut self, _value: bool) {}
    pub const fn set_display_guts(&mut self, _value: bool) {}
    pub const fn set_fit_to_page(&mut self, _value: bool) {}
    pub const fn set_row_sums_below(&mut self, _value: bool) {}
    pub const fn set_row_sums_right(&mut self, _value: bool) {}
    pub const fn set_margin(&mut self, _margin: usize, _size: f64) {}
    pub const fn set_auto_filter(&mut self, _range: &str) {}
    pub const fn set_repeating_columns(&mut self, _range: Option<&str>) {}
    pub const fn set_repeating_rows(&mut self, _range: Option<&str>) {}
    #[must_use] pub const fn get_repeating_columns(&self) -> Option<&str> { None }
    #[must_use] pub const fn get_repeating_rows(&self) -> Option<&str> { None }
    #[must_use] pub const fn get_active_cell(&self) -> Option<&str> { None }
    pub const fn set_active_cell(&mut self, _reference: &str) {}

    #[must_use] pub const fn get_cell_comments(&self) -> Vec<&str> { Vec::new() }
    #[must_use] pub const fn get_cell_comment(&self, _reference: &str) -> Option<&str> { None }
    #[must_use] pub const fn get_hyperlink_list(&self) -> Vec<&str> { Vec::new() }
    #[must_use] pub const fn get_data_validations(&self) -> Vec<&str> { Vec::new() }
    pub const fn add_validation_data(&mut self, _validation: &str) {}
    #[must_use] pub const fn get_drawing_patriarch(&self) -> Option<&str> { None }
    /// Java CSV 返回 `null`。
    #[must_use] pub const fn create_drawing_patriarch(&mut self) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    #[must_use] pub const fn set_array_formula(&mut self, _formula: &str, _range: &str) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    #[must_use] pub const fn remove_array_formula(&mut self, _row: u32, _column: u16) -> Option<()> { None }
    /// Java CSV 返回 `null`。
    #[must_use] pub const fn get_hyperlink(&self, _row: u32, _column: u16) -> Option<()> { None }
    #[must_use] pub const fn get_sheet_conditional_formatting(&self) -> Option<&str> { None }
    #[must_use] pub const fn get_data_validation_helper(&self) -> Option<&str> { None }
    #[must_use] pub const fn get_print_setup(&self) -> Option<&str> { None }
    #[must_use] pub const fn get_header(&self) -> Option<&str> { None }
    #[must_use] pub const fn get_footer(&self) -> Option<&str> { None }
}
