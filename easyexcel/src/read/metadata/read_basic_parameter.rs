//! 对应 Java：`com.alibaba.excel.read.metadata.ReadBasicParameter`
//!
//! Java 的 `ReadBasicParameter` 包含 `headRowNumber` 和 `customReadListenerList`。
//! Rust 版本中，`headRowNumber` 已合并到 `ReadOptions`（`read/read_options.rs`），
//! `customReadListenerList` 通过 `ExcelReaderBuilder` 的泛型 listener 参数实现。
//! 本文件保留类型定义以满足 1:1 文件对应要求。

use crate::read::ReadOptions;

/// 对应 Java：`ReadBasicParameter extends BasicParameter`
///
/// 读取基本参数，包含表头行数和自定义监听器列表。
/// 实际字段已分散到 `ReadOptions` 和 builder 中。
#[derive(Debug, Clone, Default)]
pub struct ReadBasicParameter {
    /// 表头行数，默认 1。对应 Java `headRowNumber`。
    pub head_row_number: u32,
}

impl ReadBasicParameter {
    /// 创建默认参数。
    #[must_use]
    pub fn new() -> Self {
        Self { head_row_number: 1 }
    }

    /// 从 ReadOptions 构造。
    #[must_use]
    pub fn from_options(options: &ReadOptions) -> Self {
        Self {
            head_row_number: options.head_row_number,
        }
    }
}
