//! CSV 单元格中立模型。

use std::fmt::Debug;

use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCellStyle, CsvRichTextString};

include!("csv_cell/csv_numeric_cell_type.rs");

include!("csv_cell/csv_cell_value.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 CSV 工作簿中的一个有类型单元格。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvCell<V: CsvCellValue = ModelCellValue> {
    column_index: u16,
    value: V,
    numeric_cell_type: Option<V::NumericCellType>,
    cell_style: Option<CsvCellStyle>,
}

impl<V: CsvCellValue> CsvCell<V> {
    /// 在零基列下标处创建空单元格。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(column_index: u16) -> Self {
        Self {
            column_index,
            value: V::EMPTY,
            numeric_cell_type: None,
            cell_style: None,
        }
    }

    /// 返回零基列下标。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn column_index(&self) -> u16 {
        self.column_index
    }

    /// 返回有类型值。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn value(&self) -> &V {
        &self.value
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 替换有类型值并刷新数字分类。
    pub fn set_value(&mut self, value: impl Into<V>) {
        self.value = value.into();
        self.numeric_cell_type = self.value.csv_numeric_cell_type();
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 存储公式文本。
    pub fn set_formula(&mut self, formula: impl Into<String>) {
        self.value = V::from_csv_formula(formula.into());
        self.numeric_cell_type = None;
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 存储 CSV 富文本包装中的纯文本。
    pub fn set_rich_text(&mut self, value: &CsvRichTextString) {
        self.value = V::from_csv_text(value.as_str().to_owned());
        self.numeric_cell_type = None;
    }

    /// 返回数字负载分类。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn numeric_cell_type(&self) -> Option<V::NumericCellType> {
        self.numeric_cell_type
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 应用 CSV 单元格样式。
    pub fn set_cell_style(&mut self, style: CsvCellStyle) {
        self.cell_style = Some(style);
    }

    /// 返回已应用的 CSV 样式。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn cell_style(&self) -> Option<&CsvCellStyle> {
        self.cell_style.as_ref()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回写入 CSV 记录的显示文本。
    #[must_use]
    pub fn display_text(&self) -> String {
        self.value.csv_display_text()
    }
}
