//! 对应 Java：`com.alibaba.excel.enums.poi.VerticalAlignmentEnum`.

/// Vertical alignment used by annotation-driven cell styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.poi.VerticalAlignmentEnum。
pub enum ExcelVerticalAlignment {
    /// Top aligned.
    Top,
    /// Vertically centered.
    Center,
    /// Bottom aligned.
    Bottom,
    /// Vertically justified.
    Justify,
    /// Vertically distributed.
    Distributed,
}
