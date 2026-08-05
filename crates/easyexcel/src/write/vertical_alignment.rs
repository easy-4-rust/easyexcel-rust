//! 垂直对齐枚举。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.style.WriteCellStyle` 中的 `verticalAlignment`。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/write/metadata/style/WriteCellStyle.java

/// 垂直单元格对齐。
///
/// 对应 Java：`com.alibaba.excel.enums.poi.VerticalAlignmentEnum`。
/// BIFF8 数值编码由 `easyexcel-xls` 内部完成。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerticalAlignment {
    /// 顶部对齐。
    Top,
    /// 垂直居中。
    Center,
    /// 底部对齐。
    Bottom,
    /// 垂直两端对齐。
    Justify,
    /// 垂直分布。
    Distributed,
}
