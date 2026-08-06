//! CSV 稀疏行模型。

use easyexcel_io::{Error, Result};
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCell, CsvCellStyle, CsvCellValue};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 具有零基行号和稀疏单元格集合的 CSV 行。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvRow<V: CsvCellValue = ModelCellValue> {
    row_index: u32,
    cells: Vec<CsvCell<V>>,
    cell_style: Option<CsvCellStyle>,
}

impl<V: CsvCellValue> CsvRow<V> {
    /// 在零基行号处创建空行。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(row_index: u32) -> Self {
        Self {
            row_index,
            cells: Vec::new(),
            cell_style: None,
        }
    }

    /// 返回零基行号。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回按创建顺序保存的单元格。
    #[must_use]
    pub fn cells(&self) -> &[CsvCell<V>] {
        &self.cells
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 按逻辑列查询单元格。
    #[must_use]
    pub fn cell(&self, column_index: u16) -> Option<&CsvCell<V>> {
        self.cells
            .iter()
            .find(|cell| cell.column_index() == column_index)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 设置行级样式。
    pub fn set_cell_style(&mut self, style: CsvCellStyle) {
        self.cell_style = Some(style);
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
        self.cells.push(CsvCell::new(column_index));
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
