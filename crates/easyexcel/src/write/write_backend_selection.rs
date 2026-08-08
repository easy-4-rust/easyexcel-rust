//! Stateful 写入器的后端选择状态。

/// Stateful `.build()` 的工作表后端状态机。
///
/// 对应 Java：`XSSFWorkbook`、`SXSSFWorkbook` 与运行期能力选择的 Rust 映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteBackendSelection {
    /// 尚未观察首批 Sheet、类型和 Handler 能力。
    AutoUndecided,
    /// 自动选择的常量内存后端；允许在后续能力冲突时晋升。
    AutoStreaming,
    /// 正在从 journal 重放到内存工作簿。
    Promoting,
    /// 自动选择或晋升后的完整内存工作簿。
    InMemory,
    /// 调用方显式要求常量内存；能力冲突必须报错。
    ExplicitStreaming,
    /// 调用方显式要求完整内存。
    ExplicitInMemory,
}

impl WriteBackendSelection {
    /// 返回当前状态是否使用严格常量内存工作表。
    #[must_use]
    pub const fn is_streaming(self) -> bool {
        matches!(self, Self::AutoStreaming | Self::ExplicitStreaming)
    }
}
