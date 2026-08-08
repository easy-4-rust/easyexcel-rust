//! CSV 单元格中立模型。

use std::fmt::Debug;

use chrono::NaiveDateTime;
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCellStyle, CsvRichTextString};

include!("csv_cell/csv_numeric_cell_type.rs");

include!("csv_cell/csv_cell_value.rs");

/// Java/POI `CellType` 的 CSV 后端中立映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CsvCellType {
    /// 尚未设置类型。
    #[default]
    None,
    /// 数字或日期序列。
    Numeric,
    /// 文本或富文本。
    String,
    /// 公式文本。
    Formula,
    /// 空单元格。
    Blank,
    /// 布尔值。
    Boolean,
    /// Excel 错误。
    Error,
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 CSV 工作簿中的一个有类型单元格。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvCell<V: CsvCellValue = ModelCellValue> {
    column_index: u16,
    row_index: u32,
    value: V,
    numeric_cell_type: Option<V::NumericCellType>,
    cell_style: Option<CsvCellStyle>,
    cell_type: CsvCellType,
    formula_data: Option<String>,
    date_value: Option<NaiveDateTime>,
    rich_text: Option<CsvRichTextString>,
}

impl<V: CsvCellValue> CsvCell<V> {
    /// 在零基列下标处创建空单元格。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn new(column_index: u16) -> Self {
        Self {
            column_index,
            row_index: 0,
            value: V::EMPTY,
            numeric_cell_type: None,
            cell_style: None,
            cell_type: CsvCellType::None,
            formula_data: None,
            date_value: None,
            rich_text: None,
        }
    }

    /// 在指定零基行列坐标创建空单元格。
    #[must_use]
    pub const fn new_at(row_index: u32, column_index: u16) -> Self {
        let mut cell = Self::new(column_index);
        cell.row_index = row_index;
        cell
    }

    /// 返回零基列下标。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn column_index(&self) -> u16 {
        self.column_index
    }
    pub const fn get_column_index(&self) -> u16 { self.column_index() }

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
        self.cell_type = if self.numeric_cell_type.is_some() {
            CsvCellType::Numeric
        } else {
            CsvCellType::String
        };
        self.formula_data = None;
        self.date_value = None;
        self.rich_text = None;
    }

    /// Java `setCellValue` 兼容入口。
    pub fn set_cell_value(&mut self, value: impl Into<V>) {
        self.set_value(value);
    }

    /// 将单元格重置为空白。
    pub fn set_blank(&mut self) {
        self.value = V::EMPTY;
        self.cell_type = CsvCellType::Blank;
        self.numeric_cell_type = None;
        self.formula_data = None;
        self.date_value = None;
        self.rich_text = None;
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 存储公式文本。
    pub fn set_formula(&mut self, formula: impl Into<String>) {
        self.value = V::from_csv_formula(formula.into());
        self.formula_data = Some(self.value.csv_display_text());
        self.cell_type = CsvCellType::Formula;
        self.numeric_cell_type = None;
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 存储 CSV 富文本包装中的纯文本。
    pub fn set_rich_text(&mut self, value: &CsvRichTextString) {
        self.value = V::from_csv_text(value.as_str().to_owned());
        self.rich_text = Some(value.clone());
        self.cell_type = CsvCellType::String;
        self.numeric_cell_type = None;
    }

    /// 写入布尔值。
    pub fn set_boolean_value(&mut self, value: bool) {
        self.value = V::from_csv_bool(value);
        self.cell_type = CsvCellType::Boolean;
        self.numeric_cell_type = None;
    }

    /// 写入数字值。
    pub fn set_number_value(&mut self, value: f64) {
        self.value = V::from_csv_number(value);
        self.cell_type = CsvCellType::Numeric;
        self.numeric_cell_type = self.value.csv_numeric_cell_type();
    }

    /// 写入日期时间并标记数字分类为日期。
    pub fn set_date_value(&mut self, value: NaiveDateTime) {
        self.date_value = Some(value);
        self.cell_type = CsvCellType::Numeric;
        self.numeric_cell_type = None;
    }

    /// 写入 Excel 错误码。
    pub fn set_cell_error_value(&mut self, value: u8) {
        self.value = V::from_csv_error(value);
        self.cell_type = CsvCellType::Error;
        self.numeric_cell_type = None;
    }

    /// 返回零基行下标。
    #[must_use]
    pub const fn row_index(&self) -> u32 { self.row_index }
    pub const fn get_row_index(&self) -> u32 { self.row_index() }

    /// 返回 Java/POI 单元格类型。
    #[must_use]
    pub const fn cell_type(&self) -> CsvCellType { self.cell_type }
    pub const fn get_cell_type(&self) -> CsvCellType { self.cell_type() }

    /// 返回公式文本。
    #[must_use]
    pub fn cell_formula(&self) -> Option<&str> { self.formula_data.as_deref() }
    pub fn get_cell_formula(&self) -> Option<&str> { self.cell_formula() }
    pub fn get_formula_data(&self) -> Option<&str> { self.cell_formula() }
    pub fn set_formula_data(&mut self, value: Option<String>) {
        match value {
            Some(value) => self.set_formula(value),
            None => {
                self.formula_data = None;
                if self.cell_type == CsvCellType::Formula { self.cell_type = CsvCellType::Blank; }
            }
        }
    }

    /// 返回数字值；非数字时按 Java CSV 默认返回 0。
    #[must_use]
    pub fn numeric_cell_value(&self) -> f64 { self.value.csv_number().unwrap_or(0.0) }
    pub fn get_numeric_cell_value(&self) -> f64 { self.numeric_cell_value() }

    /// 返回日期时间值。
    #[must_use]
    pub const fn local_date_time_cell_value(&self) -> Option<NaiveDateTime> { self.date_value }
    pub const fn get_local_date_time_cell_value(&self) -> Option<NaiveDateTime> {
        self.local_date_time_cell_value()
    }
    pub const fn get_date_cell_value(&self) -> Option<NaiveDateTime> { self.date_value }
    pub const fn get_date_value(&self) -> Option<NaiveDateTime> { self.date_value }

    /// 返回富文本值；普通字符串按需构造兼容包装。
    #[must_use]
    pub fn rich_string_cell_value(&self) -> CsvRichTextString {
        self.rich_text.clone().unwrap_or_else(|| CsvRichTextString::new(self.value.csv_display_text()))
    }
    pub fn get_rich_string_cell_value(&self) -> CsvRichTextString { self.rich_string_cell_value() }
    pub fn get_rich_text_string(&self) -> CsvRichTextString { self.rich_string_cell_value() }
    pub fn set_rich_text_string(&mut self, value: &CsvRichTextString) { self.set_rich_text(value); }

    /// 返回字符串值。
    #[must_use]
    pub fn string_cell_value(&self) -> String { self.value.csv_display_text() }
    pub fn get_string_cell_value(&self) -> String { self.string_cell_value() }
    pub fn get_string_value(&self) -> String { self.string_cell_value() }
    pub fn set_string_value(&mut self, value: impl Into<String>) {
        self.set_value(V::from_csv_text(value.into()));
    }

    /// 返回布尔值；非布尔时按 Java CSV 默认返回 false。
    #[must_use]
    pub fn boolean_cell_value(&self) -> bool { self.value.csv_bool().unwrap_or(false) }
    pub fn get_boolean_cell_value(&self) -> bool { self.boolean_cell_value() }
    pub fn get_boolean_value(&self) -> Option<bool> { self.value.csv_bool() }

    /// 返回错误码；非错误时按 Java CSV 默认返回 0。
    #[must_use]
    pub fn error_cell_value(&self) -> u8 { self.value.csv_error().unwrap_or(0) }
    pub fn get_error_cell_value(&self) -> u8 { self.error_cell_value() }

    /// Java cached formula result type 在 CSV 中等于当前类型。
    #[must_use]
    pub const fn cached_formula_result_type(&self) -> CsvCellType { self.cell_type }
    pub const fn get_cached_formula_result_type(&self) -> CsvCellType {
        self.cached_formula_result_type()
    }

    /// CSV 不支持数组公式。
    #[must_use]
    pub const fn is_part_of_array_formula_group(&self) -> bool { false }

    /// 返回数字负载分类。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn numeric_cell_type(&self) -> Option<V::NumericCellType> {
        self.numeric_cell_type
    }
    pub const fn get_numeric_cell_type(&self) -> Option<V::NumericCellType> {
        self.numeric_cell_type()
    }
    pub fn set_numeric_cell_type(&mut self, value: Option<V::NumericCellType>) {
        self.numeric_cell_type = value;
        if value.is_some() { self.cell_type = CsvCellType::Numeric; }
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
    pub const fn get_cell_style(&self) -> Option<&CsvCellStyle> { self.cell_style() }

    /// CSV 不承载批注和超链接；与 Java CSV 适配器的 no-op 语义一致。
    pub const fn remove_cell_comment(&mut self) {}
    pub const fn remove_hyperlink(&mut self) {}
    pub const fn set_as_active_cell(&mut self) {}
    pub const fn is_part_of_array_formula_group_java(&self) -> bool {
        self.is_part_of_array_formula_group()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回写入 CSV 记录的显示文本。
    #[must_use]
    pub fn display_text(&self) -> String {
        self.date_value.map_or_else(|| self.value.csv_display_text(), |value| value.format("%Y-%m-%d %H:%M:%S").to_string())
    }
}
