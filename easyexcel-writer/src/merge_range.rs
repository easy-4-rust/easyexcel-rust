//! 合并单元格范围类型。
//!
//! 对应 Java：`org.apache.poi.ss.util.CellRangeAddress`（EasyExcel 通过 `OnceAbsoluteMergeStrategy` 引用）。
//! 原文件：easyexcel-core/src/main/java/org/apache/poi/ss/util/CellRangeAddress.java

/// 绝对合并范围（行区间 × 列区间）。
///
/// 对应 Java：`org.apache.poi.ss.util.CellRangeAddress`。
/// XLSX / XLS / CSV 写入路径下，绝对合并由 `OnceAbsoluteMergeStrategy` 触发。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MergeRange {
    /// 起始行（零基）。
    pub first_row: u32,
    /// 结束行（零基，包含）。
    pub last_row: u32,
    /// 起始列（零基）。
    pub first_column: u16,
    /// 结束列（零基，包含）。
    pub last_column: u16,
}

impl MergeRange {
    /// 创建绝对合并范围。
    ///
    /// 对应 Java：`new CellRangeAddress(firstRow, lastRow, firstCol, lastCol)`。
    #[must_use]
    pub const fn new(first_row: u32, last_row: u32, first_column: u16, last_column: u16) -> Self {
        Self {
            first_row,
            last_row,
            first_column,
            last_column,
        }
    }
}
