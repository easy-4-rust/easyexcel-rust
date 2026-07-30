//! 水平对齐枚举。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.style.WriteCellStyle` 中的 `horizontalAlignment`。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/metadata/style/WriteCellStyle.java

/// 水平单元格对齐。
///
/// 对应 Java：`com.alibaba.excel.enums.poi.HorizontalAlignmentEnum`。
/// POI 的对齐码保留在 `biff8_halign` 函数中用于 BIFF8 写入。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HorizontalAlignment {
    /// Excel 的类型相关默认值。
    General,
    /// 左对齐。
    Left,
    /// 居中。
    Center,
    /// 右对齐。
    Right,
    /// 跨单元格重复内容。
    Fill,
    /// 两端对齐。
    Justify,
    /// 跨相邻单元格居中。
    CenterAcross,
}
