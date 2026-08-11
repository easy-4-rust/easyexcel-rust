use chrono::NaiveDateTime;
use easyexcel_model::CellValue as ModelCellValue;

use super::{CsvCellType, CsvCellValue};
use crate::csv::{CsvCellStyle, CsvRichTextString};

/// 对应 Java：com.alibaba.excel.metadata.csv.CsvCell。 CSV 工作簿中的一个有类型单元格。
#[derive(Debug, Clone, PartialEq)]
pub struct CsvCell<V: CsvCellValue = ModelCellValue> {
    csv_workbook_id: Option<usize>,
    csv_sheet_id: Option<usize>,
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
            csv_workbook_id: None,
            csv_sheet_id: None,
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
    /// 返回父工作簿稳定身份。对应 Java Lombok `getCsvWorkbook`。
    #[must_use]
    pub const fn get_csv_workbook(&self) -> Option<usize> {
        self.csv_workbook_id
    }
    /// 设置父工作簿稳定身份。
    pub const fn set_csv_workbook(&mut self, value: Option<usize>) {
        self.csv_workbook_id = value;
    }
    /// 返回父工作表稳定身份。对应 Java Lombok `getCsvSheet`。
    #[must_use]
    pub const fn get_csv_sheet(&self) -> Option<usize> {
        self.csv_sheet_id
    }
    /// 设置父工作表稳定身份。
    pub const fn set_csv_sheet(&mut self, value: Option<usize>) {
        self.csv_sheet_id = value;
    }
    /// 返回父行的零基行号。对应 Java Lombok `getCsvRow` 的稳定身份映射。
    #[must_use]
    pub const fn get_csv_row(&self) -> u32 {
        self.row_index
    }
    pub const fn get_column_index(&self) -> u16 {
        self.column_index()
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
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }
    pub const fn get_row_index(&self) -> u32 {
        self.row_index()
    }

    /// 返回 Java/POI 单元格类型。
    #[must_use]
    pub const fn cell_type(&self) -> CsvCellType {
        self.cell_type
    }
    pub const fn get_cell_type(&self) -> CsvCellType {
        self.cell_type()
    }

    /// 返回公式文本。
    #[must_use]
    pub fn cell_formula(&self) -> Option<&str> {
        self.formula_data.as_deref()
    }
    pub fn get_cell_formula(&self) -> Option<&str> {
        self.cell_formula()
    }
    pub fn get_formula_data(&self) -> Option<&str> {
        self.cell_formula()
    }
    pub fn set_formula_data(&mut self, value: Option<String>) {
        match value {
            Some(value) => self.set_formula(value),
            None => {
                self.formula_data = None;
                if self.cell_type == CsvCellType::Formula {
                    self.cell_type = CsvCellType::Blank;
                }
            }
        }
    }

    /// 返回数字值；非数字时按 Java CSV 默认返回 0。
    #[must_use]
    pub fn numeric_cell_value(&self) -> f64 {
        self.value.csv_number().unwrap_or(0.0)
    }
    pub fn get_numeric_cell_value(&self) -> f64 {
        self.numeric_cell_value()
    }

    /// 返回日期时间值。
    #[must_use]
    pub const fn local_date_time_cell_value(&self) -> Option<NaiveDateTime> {
        self.date_value
    }
    pub const fn get_local_date_time_cell_value(&self) -> Option<NaiveDateTime> {
        self.local_date_time_cell_value()
    }
    pub const fn get_date_cell_value(&self) -> Option<NaiveDateTime> {
        self.date_value
    }
    pub const fn get_date_value(&self) -> Option<NaiveDateTime> {
        self.date_value
    }

    /// 返回富文本值；普通字符串按需构造兼容包装。
    #[must_use]
    pub fn rich_string_cell_value(&self) -> CsvRichTextString {
        self.rich_text
            .clone()
            .unwrap_or_else(|| CsvRichTextString::new(self.value.csv_display_text()))
    }
    pub fn get_rich_string_cell_value(&self) -> CsvRichTextString {
        self.rich_string_cell_value()
    }
    pub fn get_rich_text_string(&self) -> CsvRichTextString {
        self.rich_string_cell_value()
    }
    pub fn set_rich_text_string(&mut self, value: &CsvRichTextString) {
        self.set_rich_text(value);
    }

    /// 返回字符串值。
    #[must_use]
    pub fn string_cell_value(&self) -> String {
        self.value.csv_display_text()
    }
    pub fn get_string_cell_value(&self) -> String {
        self.string_cell_value()
    }
    pub fn get_string_value(&self) -> String {
        self.string_cell_value()
    }
    pub fn set_string_value(&mut self, value: impl Into<String>) {
        self.set_value(V::from_csv_text(value.into()));
    }

    /// 返回布尔值；非布尔时按 Java CSV 默认返回 false。
    #[must_use]
    pub fn boolean_cell_value(&self) -> bool {
        self.value.csv_bool().unwrap_or(false)
    }
    pub fn get_boolean_cell_value(&self) -> bool {
        self.boolean_cell_value()
    }
    pub fn get_boolean_value(&self) -> Option<bool> {
        self.value.csv_bool()
    }

    /// 返回错误码；非错误时按 Java CSV 默认返回 0。
    #[must_use]
    pub fn error_cell_value(&self) -> u8 {
        self.value.csv_error().unwrap_or(0)
    }
    pub fn get_error_cell_value(&self) -> u8 {
        self.error_cell_value()
    }

    /// Java cached formula result type 在 CSV 中等于当前类型。
    #[must_use]
    pub const fn cached_formula_result_type(&self) -> CsvCellType {
        self.cell_type
    }
    pub const fn get_cached_formula_result_type(&self) -> CsvCellType {
        self.cached_formula_result_type()
    }

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
        if value.is_some() {
            self.cell_type = CsvCellType::Numeric;
        }
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
    pub const fn get_cell_style(&self) -> Option<&CsvCellStyle> {
        self.cell_style()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回写入 CSV 记录的显示文本。
    #[must_use]
    pub fn display_text(&self) -> String {
        self.date_value.map_or_else(
            || self.value.csv_display_text(),
            |value| value.format("%Y-%m-%d %H:%M:%S").to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::CsvNumericCellType;
    use crate::csv::CsvRichTextString;
    use chrono::NaiveDate;
    use easyexcel_model::CellValue as ModelCellValue;

    type TestCell = CsvCell<ModelCellValue>;

    #[test]
    fn new_creates_empty_cell_at_column() {
        let cell = TestCell::new(5);
        assert_eq!(cell.column_index(), 5);
        assert_eq!(cell.row_index(), 0);
        assert_eq!(cell.cell_type(), CsvCellType::None);
        assert_eq!(*cell.value(), ModelCellValue::Empty);
    }

    #[test]
    fn new_at_sets_row_and_column() {
        let cell = TestCell::new_at(10, 3);
        assert_eq!(cell.row_index(), 10);
        assert_eq!(cell.column_index(), 3);
    }

    #[test]
    fn set_value_text() {
        let mut cell = TestCell::new(0);
        cell.set_value(ModelCellValue::Text("hello".to_string()));
        assert_eq!(*cell.value(), ModelCellValue::Text("hello".into()));
        assert_eq!(cell.cell_type(), CsvCellType::String);
    }

    #[test]
    fn set_value_number() {
        let mut cell = TestCell::new(0);
        cell.set_value(ModelCellValue::Number(42.0));
        assert_eq!(*cell.value(), ModelCellValue::Number(42.0));
        assert_eq!(cell.cell_type(), CsvCellType::Numeric);
        assert!(cell.numeric_cell_type().is_some());
    }

    #[test]
    fn set_cell_value_delegates_to_set_value() {
        let mut cell = TestCell::new(0);
        cell.set_string_value("test");
        assert_eq!(cell.string_cell_value(), "test");
    }

    #[test]
    fn set_blank_resets_cell() {
        let mut cell = TestCell::new(0);
        cell.set_string_value("data");
        cell.set_blank();
        assert_eq!(*cell.value(), ModelCellValue::Empty);
        assert_eq!(cell.cell_type(), CsvCellType::Blank);
        assert!(cell.numeric_cell_type().is_none());
        assert!(cell.cell_formula().is_none());
    }

    #[test]
    fn set_formula() {
        let mut cell = TestCell::new(0);
        cell.set_formula("SUM(A1:A10)");
        assert_eq!(cell.cell_type(), CsvCellType::Formula);
        assert_eq!(cell.cell_formula(), Some("SUM(A1:A10)"));
    }

    #[test]
    fn set_rich_text() {
        let mut cell = TestCell::new(0);
        let rich = CsvRichTextString::new("rich content");
        cell.set_rich_text(&rich);
        assert_eq!(cell.cell_type(), CsvCellType::String);
        assert_eq!(cell.string_cell_value(), "rich content");
        assert_eq!(cell.rich_string_cell_value(), rich);
    }

    #[test]
    fn set_boolean_value() {
        let mut cell = TestCell::new(0);
        cell.set_boolean_value(true);
        assert_eq!(cell.cell_type(), CsvCellType::Boolean);
        assert!(cell.boolean_cell_value());
    }

    #[test]
    fn set_boolean_value_false() {
        let mut cell = TestCell::new(0);
        cell.set_boolean_value(false);
        assert!(!cell.boolean_cell_value());
    }

    #[test]
    fn set_number_value() {
        let mut cell = TestCell::new(0);
        cell.set_number_value(3.14);
        assert_eq!(cell.cell_type(), CsvCellType::Numeric);
        assert!((cell.numeric_cell_value() - 3.14).abs() < f64::EPSILON);
    }

    #[test]
    fn set_date_value() {
        let mut cell = TestCell::new(0);
        let dt = NaiveDate::from_ymd_opt(2024, 1, 15)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();
        cell.set_date_value(dt);
        assert_eq!(cell.cell_type(), CsvCellType::Numeric);
        assert_eq!(cell.local_date_time_cell_value(), Some(dt));
    }

    #[test]
    fn set_cell_error_value() {
        let mut cell = TestCell::new(0);
        cell.set_cell_error_value(0); // #NULL!
        assert_eq!(cell.cell_type(), CsvCellType::Error);
    }

    #[test]
    fn set_string_value() {
        let mut cell = TestCell::new(0);
        cell.set_string_value("text data");
        assert_eq!(cell.string_cell_value(), "text data");
    }

    #[test]
    fn getters_alias_correctly() {
        let mut cell = TestCell::new_at(7, 3);
        cell.set_number_value(99.0);
        assert_eq!(cell.get_row_index(), 7);
        assert_eq!(cell.get_column_index(), 3);
        assert_eq!(cell.get_cell_type(), CsvCellType::Numeric);
        assert!((cell.get_numeric_cell_value() - 99.0).abs() < f64::EPSILON);
    }

    #[test]
    fn csv_workbook_and_sheet_ids() {
        let mut cell = TestCell::new(0);
        assert!(cell.get_csv_workbook().is_none());
        cell.set_csv_workbook(Some(42));
        assert_eq!(cell.get_csv_workbook(), Some(42));
        assert!(cell.get_csv_sheet().is_none());
        cell.set_csv_sheet(Some(7));
        assert_eq!(cell.get_csv_sheet(), Some(7));
    }

    #[test]
    fn get_csv_row_returns_row_index() {
        let cell = TestCell::new_at(5, 0);
        assert_eq!(cell.get_csv_row(), 5);
    }

    #[test]
    fn boolean_cell_value_default_false() {
        let cell = TestCell::new(0);
        assert!(!cell.boolean_cell_value());
        assert_eq!(cell.get_boolean_value(), None);
    }

    #[test]
    fn numeric_cell_value_default_zero() {
        let cell = TestCell::new(0);
        assert!((cell.numeric_cell_value() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn error_cell_value_default_zero() {
        let cell = TestCell::new(0);
        assert_eq!(cell.error_cell_value(), 0);
    }

    #[test]
    fn rich_string_cell_value_fallback() {
        let mut cell = TestCell::new(0);
        cell.set_string_value("plain");
        // 没有显式 rich_text，应回退到 value 的 display text
        let rich = cell.rich_string_cell_value();
        assert_eq!(rich.as_str(), "plain");
    }

    #[test]
    fn cached_formula_result_type_matches_cell_type() {
        let mut cell = TestCell::new(0);
        cell.set_number_value(1.0);
        assert_eq!(cell.cached_formula_result_type(), CsvCellType::Numeric);
    }

    #[test]
    fn set_formula_data_none_clears_formula() {
        let mut cell = TestCell::new(0);
        cell.set_formula("A1");
        assert_eq!(cell.cell_type(), CsvCellType::Formula);
        cell.set_formula_data(None);
        assert!(cell.cell_formula().is_none());
        assert_eq!(cell.cell_type(), CsvCellType::Blank);
    }

    #[test]
    fn set_formula_data_some_sets_formula() {
        let mut cell = TestCell::new(0);
        cell.set_formula_data(Some("B1+C1".to_string()));
        assert_eq!(cell.cell_type(), CsvCellType::Formula);
        assert!(cell.cell_formula().is_some());
    }

    #[test]
    fn set_cell_style() {
        let mut cell = TestCell::new(0);
        assert!(cell.cell_style().is_none());
        let style = CsvCellStyle::new(5);
        cell.set_cell_style(style);
        assert!(cell.cell_style().is_some());
        assert_eq!(cell.cell_style().unwrap().index(), 5);
    }

    #[test]
    fn set_numeric_cell_type_some_switches_to_numeric() {
        let mut cell = TestCell::new(0);
        cell.set_string_value("text");
        assert_eq!(cell.cell_type(), CsvCellType::String);
        cell.set_numeric_cell_type(Some(CsvNumericCellType::Number));
        assert_eq!(cell.cell_type(), CsvCellType::Numeric);
    }

    #[test]
    fn display_text_with_date() {
        let mut cell = TestCell::new(0);
        let dt = NaiveDate::from_ymd_opt(2024, 6, 15)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        cell.set_date_value(dt);
        assert_eq!(cell.display_text(), "2024-06-15 12:00:00");
    }

    #[test]
    fn display_text_without_date() {
        let mut cell = TestCell::new(0);
        cell.set_string_value("hello");
        assert_eq!(cell.display_text(), "hello");
    }

    #[test]
    fn rich_text_string_setter_and_getter() {
        let mut cell = TestCell::new(0);
        let rich = CsvRichTextString::new("rich text");
        cell.set_rich_text_string(&rich);
        assert_eq!(cell.get_rich_text_string(), rich);
        assert_eq!(cell.get_rich_string_cell_value(), rich);
    }

    #[test]
    fn local_date_time_aliases() {
        let mut cell = TestCell::new(0);
        let dt = NaiveDate::from_ymd_opt(2024, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        cell.set_date_value(dt);
        assert_eq!(cell.get_local_date_time_cell_value(), Some(dt));
        assert_eq!(cell.get_date_cell_value(), Some(dt));
        assert_eq!(cell.get_date_value(), Some(dt));
    }
}
