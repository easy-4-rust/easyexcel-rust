//! CSV 单元格类型枚举。
//!
//! 对应 Java：POI `CellType` 的 CSV 后端中立映射。
//! 从 `csv_cell.rs` 拆分而来，遵循"一个 .rs 文件只对应一个 Java 对象"规范。

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
