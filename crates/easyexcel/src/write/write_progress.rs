//! 写入进度跟踪类型。
//!
//! 对应 Java：内部进度跟踪结构。
//! 原文件：easyexcel-writer 内部进度跟踪。

/// 写入进度，用于跟踪写入到工作表的行位置。
///
/// 对应 Java：内部 `WriteProgress` 结构。
/// 由 `ExcelWriteAddExecutor` 和状态化 `ExcelWriter` 路径使用，
/// 它们都委托给 `append_rows_to_worksheet`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteProgress {
    /// 下一个零基物理工作表行号。
    pub next_row: u32,
    /// 下一个零基数据行索引（不包括表头行）。
    pub next_data_index: usize,
}
