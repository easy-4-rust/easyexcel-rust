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
    #[must_use]
    pub const fn get_csv_workbook(&self) -> Option<usize> {
        self.csv_workbook_id
    }
    /// 设置父工作簿稳定身份，避免 Rust 自引用结构。
    pub fn set_csv_workbook(&mut self, value: Option<usize>) {
        self.csv_workbook_id = value;
        for row in &mut self.row_cache {
            row.set_csv_workbook(value);
            row.set_csv_sheet(Some(self.identity));
        }
    }
    /// 返回输出缓冲。对应 Java Lombok `getOut`。
    #[must_use]
    pub fn get_out(&self) -> &str {
        &self.out
    }
    /// 设置输出缓冲。对应 Java Lombok `setOut`。
    pub fn set_out(&mut self, value: impl Into<String>) {
        self.out = value.into();
    }
    /// 返回 CSV printer 初始化状态的后端中立映射。
    #[must_use]
    pub const fn get_csv_printer(&self) -> bool {
        self.csv_printer_initialized
    }
    /// 设置 CSV printer 初始化状态的后端中立映射。
    pub const fn set_csv_printer(&mut self, value: bool) {
        self.csv_printer_initialized = value;
    }

    /// 返回 Java `CsvSheet#getRowCacheCount` 对应的有界缓存行数。
    #[must_use]
    pub const fn row_cache_count(&self) -> usize {
        self.row_cache_count
    }
    pub const fn get_row_cache_count(&self) -> usize {
        self.row_cache_count()
    }

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
    pub const fn get_last_row_index(&self) -> Option<u32> {
        self.last_row_index()
    }

    /// 返回 Java `CsvSheet#getFirstRowNum` 的零基语义；空表返回 `None`。
    #[must_use]
    pub const fn first_row_num(&self) -> Option<u32> {
        if self.last_row_index.is_some() {
            Some(0)
        } else {
            None
        }
    }
    pub const fn get_first_row_num(&self) -> Option<u32> {
        self.first_row_num()
    }

    /// 返回 Java `CsvSheet#getLastRowNum` 的零基语义；空表返回 `None`。
    #[must_use]
    pub const fn last_row_num(&self) -> Option<u32> {
        self.last_row_index
    }
    pub const fn get_last_row_num(&self) -> Option<u32> {
        self.last_row_num()
    }

    /// 返回当前仍驻留在有界窗口内的物理行数。
    #[must_use]
    pub fn physical_number_of_rows(&self) -> usize {
        self.row_cache.len()
    }
    pub fn get_physical_number_of_rows(&self) -> usize {
        self.physical_number_of_rows()
    }

    /// 返回当前缓存窗口，语义对应 Java Lombok `getRowCache`。
    #[must_use]
    pub fn row_cache(&self) -> &VecDeque<CsvRow<V>> {
        &self.row_cache
    }
    pub fn get_row_cache(&self) -> &VecDeque<CsvRow<V>> {
        self.row_cache()
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_model::CellValue as ModelCellValue;

    type TestSheet = CsvSheet<ModelCellValue>;

    #[test]
    fn new_sheet_has_name_and_defaults() {
        let sheet = TestSheet::new("MySheet");
        assert_eq!(sheet.name(), "MySheet");
        assert_eq!(sheet.get_sheet_name(), "MySheet");
        assert_eq!(sheet.row_cache_count(), 100);
        assert!(sheet.last_row_index().is_none());
        assert!(sheet.last_row_num().is_none());
        assert!(sheet.first_row_num().is_none());
        assert_eq!(sheet.physical_number_of_rows(), 0);
    }

    #[test]
    fn identity_is_unique() {
        let s1 = TestSheet::new("A");
        let s2 = TestSheet::new("B");
        assert_ne!(s1.identity(), s2.identity());
    }

    #[test]
    fn try_create_row_in_order() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        sheet.try_create_row(1).unwrap();
        sheet.try_create_row(2).unwrap();
        assert_eq!(sheet.last_row_index(), Some(2));
        assert_eq!(sheet.last_row_num(), Some(2));
        assert_eq!(sheet.first_row_num(), Some(0));
        assert_eq!(sheet.physical_number_of_rows(), 3);
    }

    #[test]
    fn try_create_row_out_of_order_errors() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        assert!(sheet.try_create_row(5).is_err());
    }

    #[test]
    fn create_row_delegates() {
        let mut sheet = TestSheet::new("S");
        sheet.create_row(0).unwrap();
        assert_eq!(sheet.physical_number_of_rows(), 1);
    }

    #[test]
    fn row_lookup() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        let row = sheet.row(0).unwrap();
        assert_eq!(row.row_index(), 0);
    }

    #[test]
    fn row_not_found_errors() {
        let sheet = TestSheet::new("S");
        assert!(sheet.row(0).is_err());
    }

    #[test]
    fn get_row_alias() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        assert!(sheet.get_row(0).is_ok());
    }

    #[test]
    fn take_last_row() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        sheet.try_create_row(1).unwrap();
        let row = sheet.take_last_row().unwrap();
        assert_eq!(row.row_index(), 1);
        assert_eq!(sheet.physical_number_of_rows(), 1);
    }

    #[test]
    fn take_last_row_empty_returns_none() {
        let mut sheet = TestSheet::new("S");
        assert!(sheet.take_last_row().is_none());
    }

    #[test]
    fn rows_iterator() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        sheet.try_create_row(1).unwrap();
        assert_eq!(sheet.rows().count(), 2);
    }

    #[test]
    fn remove_row_unsupported() {
        let mut sheet = TestSheet::new("S");
        assert!(sheet.remove_row(0).is_err());
    }

    #[test]
    fn set_row_cache_count_clamps_to_one() {
        let mut sheet = TestSheet::new("S");
        sheet.set_row_cache_count(0);
        assert_eq!(sheet.row_cache_count(), 1);
    }

    #[test]
    fn set_row_cache_count_drains_excess() {
        let mut sheet = TestSheet::new("S");
        for i in 0..5 {
            sheet.try_create_row(i).unwrap();
        }
        assert_eq!(sheet.physical_number_of_rows(), 5);
        sheet.set_row_cache_count(3);
        assert_eq!(sheet.row_cache_count(), 3);
        assert_eq!(sheet.physical_number_of_rows(), 3);
    }

    #[test]
    fn print_data_returns_excess() {
        let mut sheet = TestSheet::new("S");
        sheet.set_row_cache_count(2);
        sheet.try_create_row(0).unwrap();
        sheet.try_create_row(1).unwrap();
        // 第三行触发 print_data
        sheet.try_create_row(2).unwrap();
        let flushed = sheet.print_data();
        assert_eq!(flushed.len(), 1);
    }

    #[test]
    fn print_data_returns_empty_when_under_limit() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        assert!(sheet.print_data().is_empty());
    }

    #[test]
    fn drain_flushable_rows() {
        let mut sheet = TestSheet::new("S");
        sheet.set_row_cache_count(2);
        for i in 0..5 {
            sheet.try_create_row(i).unwrap();
        }
        let drained = sheet.drain_flushable_rows();
        assert_eq!(drained.len(), 3); // 5 - 2 = 3
        assert_eq!(sheet.physical_number_of_rows(), 2);
    }

    #[test]
    fn set_next_row_index() {
        let mut sheet = TestSheet::new("S");
        sheet.set_next_row_index(5);
        assert_eq!(sheet.last_row_index(), Some(4));
    }

    #[test]
    fn csv_workbook_id_propagation() {
        let mut sheet = TestSheet::new("S");
        sheet.set_csv_workbook(Some(42));
        assert_eq!(sheet.get_csv_workbook(), Some(42));
        sheet.try_create_row(0).unwrap();
        assert_eq!(
            sheet.row_cache().front().unwrap().get_csv_workbook(),
            Some(42)
        );
        assert_eq!(
            sheet.row_cache().front().unwrap().get_csv_sheet(),
            Some(sheet.identity())
        );
    }

    #[test]
    fn out_buffer() {
        let mut sheet = TestSheet::new("S");
        assert_eq!(sheet.get_out(), "");
        sheet.set_out("buffer content");
        assert_eq!(sheet.get_out(), "buffer content");
    }

    #[test]
    fn csv_printer_flag() {
        let mut sheet = TestSheet::new("S");
        assert!(!sheet.get_csv_printer());
        sheet.set_csv_printer(true);
        assert!(sheet.get_csv_printer());
    }

    #[test]
    fn row_cache_mut_returns_mutable() {
        let mut sheet = TestSheet::new("S");
        sheet.try_create_row(0).unwrap();
        let cache = sheet.row_cache_mut();
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn set_row_cache_replaces_and_propagates() {
        let mut sheet = TestSheet::new("S");
        let mut deque = VecDeque::new();
        deque.push_back(CsvRow::new(0));
        deque.push_back(CsvRow::new(1));
        sheet.set_csv_workbook(Some(99));
        sheet.set_row_cache(deque);
        assert_eq!(sheet.physical_number_of_rows(), 2);
        assert_eq!(sheet.last_row_index(), Some(1));
        for row in sheet.row_cache() {
            assert_eq!(row.get_csv_workbook(), Some(99));
            assert_eq!(row.get_csv_sheet(), Some(sheet.identity()));
        }
    }
}
