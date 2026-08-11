use std::fmt::Debug;

use easyexcel_model::CellValue as ModelCellValue;

use super::CsvNumericCellType;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 可存入 [`CsvCell`] 的值契约。
///
/// `EasyExcel` 门面通过此契约接入其 Java 风格 `CellValue`，基础 crate
/// 默认实现则使用 [`easyexcel_model::CellValue`]。
pub trait CsvCellValue: Debug + Clone + PartialEq + Sized {
    /// 与值类型配套的数字分类。
    type NumericCellType: Debug + Clone + Copy + PartialEq + Eq;

    /// 空单元格常量。
    const EMPTY: Self;

    /// 从普通文本创建值。
    fn from_csv_text(value: String) -> Self;

    /// 从公式文本创建值。
    fn from_csv_formula(value: String) -> Self;

    /// 从布尔值创建单元格值。
    fn from_csv_bool(value: bool) -> Self;

    /// 从数字创建单元格值。
    fn from_csv_number(value: f64) -> Self;

    /// 从 Excel 错误码创建单元格值。
    fn from_csv_error(value: u8) -> Self;

    /// 返回底层数字。
    fn csv_number(&self) -> Option<f64>;

    /// 返回底层布尔值。
    fn csv_bool(&self) -> Option<bool>;

    /// 返回错误码。
    fn csv_error(&self) -> Option<u8>;

    /// 返回数字负载分类。
    fn csv_numeric_cell_type(&self) -> Option<Self::NumericCellType>;

    /// 返回写入 CSV 记录的显示文本。
    fn csv_display_text(&self) -> String;
}

impl CsvCellValue for ModelCellValue {
    type NumericCellType = CsvNumericCellType;

    const EMPTY: Self = Self::Empty;

    fn from_csv_text(value: String) -> Self {
        Self::Text(value)
    }

    fn from_csv_formula(value: String) -> Self {
        Self::Text(value)
    }

    fn from_csv_bool(value: bool) -> Self {
        Self::Bool(value)
    }

    fn from_csv_number(value: f64) -> Self {
        Self::Number(value)
    }

    fn from_csv_error(value: u8) -> Self {
        Self::Error(easyexcel_model::CellError::from_biff_code(value))
    }

    fn csv_number(&self) -> Option<f64> {
        if let Self::Number(value) = self { Some(*value) } else { None }
    }

    fn csv_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self { Some(*value) } else { None }
    }

    fn csv_error(&self) -> Option<u8> {
        if let Self::Error(value) = self { Some(value.biff_code()) } else { None }
    }

    fn csv_numeric_cell_type(&self) -> Option<Self::NumericCellType> {
        matches!(self, Self::Number(_)).then_some(CsvNumericCellType::Number)
    }

    fn csv_display_text(&self) -> String {
        self.to_display_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_model::CellValue;

    #[test]
    fn from_csv_text() {
        let v = ModelCellValue::from_csv_text("hello".to_string());
        assert_eq!(v, CellValue::Text("hello".into()));
    }

    #[test]
    fn from_csv_formula_stores_as_text() {
        let v = ModelCellValue::from_csv_formula("A1+B1".to_string());
        assert_eq!(v, CellValue::Text("A1+B1".into()));
    }

    #[test]
    fn from_csv_bool_true() {
        let v = ModelCellValue::from_csv_bool(true);
        assert_eq!(v, CellValue::Bool(true));
    }

    #[test]
    fn from_csv_bool_false() {
        let v = ModelCellValue::from_csv_bool(false);
        assert_eq!(v, CellValue::Bool(false));
    }

    #[test]
    fn from_csv_number() {
        let v = ModelCellValue::from_csv_number(42.5);
        assert_eq!(v, CellValue::Number(42.5));
    }

    #[test]
    fn csv_number_some() {
        let v = ModelCellValue::Number(3.14);
        assert_eq!(v.csv_number(), Some(3.14));
    }

    #[test]
    fn csv_number_none_for_text() {
        let v = ModelCellValue::Text("not a number".into());
        assert_eq!(v.csv_number(), None);
    }

    #[test]
    fn csv_bool_some() {
        let v = ModelCellValue::Bool(true);
        assert_eq!(v.csv_bool(), Some(true));
    }

    #[test]
    fn csv_bool_none_for_number() {
        let v = ModelCellValue::Number(1.0);
        assert_eq!(v.csv_bool(), None);
    }

    #[test]
    fn csv_numeric_cell_type_for_number() {
        let v = ModelCellValue::Number(1.0);
        assert_eq!(v.csv_numeric_cell_type(), Some(CsvNumericCellType::Number));
    }

    #[test]
    fn csv_numeric_cell_type_none_for_text() {
        let v = ModelCellValue::Text("abc".into());
        assert_eq!(v.csv_numeric_cell_type(), None);
    }

    #[test]
    fn csv_display_text_variants() {
        assert_eq!(ModelCellValue::Empty.csv_display_text(), "");
        assert_eq!(ModelCellValue::Text("hi".into()).csv_display_text(), "hi");
        assert_eq!(ModelCellValue::Bool(true).csv_display_text(), "TRUE");
        assert_eq!(ModelCellValue::Bool(false).csv_display_text(), "FALSE");
        assert_eq!(ModelCellValue::Number(42.0).csv_display_text(), "42");
    }

    #[test]
    fn empty_constant() {
        assert_eq!(ModelCellValue::EMPTY, CellValue::Empty);
    }
}
