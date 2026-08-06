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

    fn csv_numeric_cell_type(&self) -> Option<Self::NumericCellType> {
        matches!(self, Self::Number(_)).then_some(CsvNumericCellType::Number)
    }

    fn csv_display_text(&self) -> String {
        self.to_display_string()
    }
}

