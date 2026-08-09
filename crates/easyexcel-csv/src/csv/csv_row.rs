//! CSV 稀疏行模型。

use easyexcel_io::{Error, Result};
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCell, CsvCellStyle, CsvCellValue};

/// 对应 Java：com.alibaba.excel.metadata.csv.CsvRow。 具有零基行号和稀疏单元格集合的 CSV 行。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvRow<V: CsvCellValue = ModelCellValue> {
    csv_workbook_id: Option<usize>,
    csv_sheet_id: Option<usize>,
    row_index: u32,
    cells: Vec<CsvCell<V>>,
    cell_style: Option<CsvCellStyle>,
    height_twips: u16,
    zero_height: bool,
}

impl<V: CsvCellValue> CsvRow<V> {
    /// 在零基行号处创建空行。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(row_index: u32) -> Self {
        Self {
            csv_workbook_id: None,
            csv_sheet_id: None,
            row_index,
            cells: Vec::new(),
            cell_style: None,
            height_twips: 0,
            zero_height: false,
        }
    }

    /// 返回零基行号。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }
    /// 返回父工作簿稳定身份。对应 Java Lombok `getCsvWorkbook`。
    #[must_use] pub const fn get_csv_workbook(&self) -> Option<usize> { self.csv_workbook_id }
    /// 设置父工作簿稳定身份。
    pub fn set_csv_workbook(&mut self, value: Option<usize>) {
        self.csv_workbook_id = value;
        for cell in &mut self.cells {
            cell.set_csv_workbook(value);
        }
    }
    /// 返回父工作表稳定身份。对应 Java Lombok `getCsvSheet`。
    #[must_use]
    pub const fn get_csv_sheet(&self) -> Option<usize> { self.csv_sheet_id }
    /// 设置父工作表稳定身份并传播给已有单元格。
    pub fn set_csv_sheet(&mut self, value: Option<usize>) {
        self.csv_sheet_id = value;
        for cell in &mut self.cells {
            cell.set_csv_sheet(value);
        }
    }

    /// 设置零基行号，语义对应 Java Lombok `setRowIndex` / `setRowNum`。
    pub const fn set_row_index(&mut self, row_index: u32) {
        self.row_index = row_index;
    }

    /// 设置零基行号。对应 Java：`CsvRow#setRowNum`。
    pub const fn set_row_num(&mut self, row_num: u32) {
        self.set_row_index(row_num);
    }

    /// Java `Row#getRowNum` 兼容别名。
    #[must_use]
    pub const fn row_num(&self) -> u32 {
        self.row_index
    }
    pub const fn get_row_num(&self) -> u32 { self.row_num() }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回按创建顺序保存的单元格。
    #[must_use]
    pub fn cells(&self) -> &[CsvCell<V>] {
        &self.cells
    }

    /// 返回单元格列表。对应 Java Lombok：`CsvRow#getCellList`。
    #[must_use]
    pub fn get_cell_list(&self) -> &[CsvCell<V>] {
        self.cells()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按逻辑列查询单元格。
    #[must_use]
    pub fn cell(&self, column_index: u16) -> Option<&CsvCell<V>> {
        self.cells
            .iter()
            .find(|cell| cell.column_index() == column_index)
    }
    pub fn get_cell(&self, column_index: u16) -> Option<&CsvCell<V>> { self.cell(column_index) }

    /// 返回指定列的可变单元格。
    pub fn cell_mut(&mut self, column_index: u16) -> Option<&mut CsvCell<V>> {
        self.cells
            .iter_mut()
            .find(|cell| cell.column_index() == column_index)
    }

    /// Java `Row#getPhysicalNumberOfCells` 兼容入口。
    #[must_use]
    pub fn physical_number_of_cells(&self) -> usize {
        self.cells.len()
    }
    pub fn get_physical_number_of_cells(&self) -> usize { self.physical_number_of_cells() }

    /// 返回首个已创建列号；空行返回 `None`。
    #[must_use]
    pub fn first_cell_num(&self) -> Option<u16> {
        self.cells.iter().map(CsvCell::column_index).min()
    }
    pub fn get_first_cell_num(&self) -> Option<u16> { self.first_cell_num() }

    /// 返回最后单元格后一列；空行返回 `None`。
    #[must_use]
    pub fn last_cell_num(&self) -> Option<u16> {
        self.cells
            .iter()
            .map(CsvCell::column_index)
            .max()
            .map(|column| column.saturating_add(1))
    }
    pub fn get_last_cell_num(&self) -> Option<u16> { self.last_cell_num() }

    /// 删除指定列单元格。
    pub fn remove_cell(&mut self, column_index: u16) -> Option<CsvCell<V>> {
        let index = self
            .cells
            .iter()
            .position(|cell| cell.column_index() == column_index)?;
        Some(self.cells.remove(index))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 设置行级样式。
    pub fn set_cell_style(&mut self, style: CsvCellStyle) {
        self.cell_style = Some(style);
    }

    /// 设置行样式。对应 Java：`CsvRow#setRowStyle`。
    pub fn set_row_style(&mut self, style: CsvCellStyle) {
        self.set_cell_style(style);
    }

    /// 返回行级样式。
    #[must_use]
    pub const fn cell_style(&self) -> Option<&CsvCellStyle> {
        self.cell_style.as_ref()
    }
    pub const fn get_row_style(&self) -> Option<&CsvCellStyle> { self.cell_style() }

    /// 返回行是否具有格式。
    #[must_use]
    pub const fn is_formatted(&self) -> bool {
        self.cell_style.is_some()
    }
    pub const fn get_zero_height(&self) -> bool { self.zero_height() }

    /// 设置行高，单位为 twip（1/20 point）。
    pub const fn set_height(&mut self, height_twips: u16) {
        self.height_twips = height_twips;
    }

    /// 返回 twip 行高。
    #[must_use]
    pub const fn height(&self) -> u16 {
        self.height_twips
    }
    pub const fn get_height(&self) -> u16 { self.height() }

    /// 设置点数行高。
    pub fn set_height_in_points(&mut self, height_points: f32) {
        self.height_twips = (height_points.max(0.0) * 20.0).round().min(f32::from(u16::MAX)) as u16;
    }

    /// 返回点数行高。
    #[must_use]
    pub fn height_in_points(&self) -> f32 {
        f32::from(self.height_twips) / 20.0
    }
    pub fn get_height_in_points(&self) -> f32 { self.height_in_points() }

    /// 设置零高度隐藏标志。
    pub const fn set_zero_height(&mut self, zero_height: bool) {
        self.zero_height = zero_height;
    }

    /// 返回零高度隐藏标志。
    #[must_use]
    pub const fn zero_height(&self) -> bool {
        self.zero_height
    }

    /// Java `cellIterator()` 兼容入口。
    pub fn cell_iterator(&self) -> impl Iterator<Item = &CsvCell<V>> { self.iter() }

    /// CSV 不维护分组层级。对应 Java：`CsvRow#getOutlineLevel`。
    #[must_use]
    pub const fn get_outline_level(&self) -> i32 { 0 }

    /// Java CSV 实现为空操作，不移动单元格。
    pub const fn shift_cells_right(
        &mut self,
        _first_shift_column_index: u16,
        _last_shift_column_index: u16,
        _step: u16,
    ) {
    }

    /// Java CSV 实现为空操作，不移动单元格。
    pub const fn shift_cells_left(
        &mut self,
        _first_shift_column_index: u16,
        _last_shift_column_index: u16,
        _step: u16,
    ) {
    }

    /// 返回单元格迭代器，语义对应 Java `cellIterator` / `iterator`。
    pub fn iter(&self) -> impl Iterator<Item = &CsvCell<V>> {
        self.cells.iter()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 创建唯一列单元格。
    ///
    /// # Errors
    ///
    /// 同一列已经存在单元格时返回 CSV 格式错误。
    pub fn try_create_cell(&mut self, column_index: u16) -> Result<&mut CsvCell<V>> {
        if self
            .cells
            .iter()
            .any(|cell| cell.column_index() == column_index)
        {
            return Err(Error::Csv(format!(
                "CSV cell already exists at row {}, column {column_index}",
                self.row_index
            )));
        }
        let mut cell = CsvCell::new_at(self.row_index, column_index);
        cell.set_csv_workbook(self.csv_workbook_id);
        cell.set_csv_sheet(self.csv_sheet_id);
        self.cells.push(cell);
        self.cells
            .last_mut()
            .ok_or_else(|| Error::Csv("CSV cell append produced no cell".to_owned()))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 构建包含 `width` 列的稠密 CSV 记录。
    #[must_use]
    pub fn into_record(self, width: usize) -> Vec<String> {
        let mut record = vec![String::new(); width];
        for cell in self.cells {
            let index = usize::from(cell.column_index());
            if let Some(slot) = record.get_mut(index) {
                *slot = cell.display_text();
            }
        }
        record
    }
}
