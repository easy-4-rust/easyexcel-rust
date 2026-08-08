//! 写入处理器的后端能力声明。

/// 描述处理器对工作表后端的最低访问能力要求。
///
/// 对应 Java：POI `SXSSF` 行窗口与 `XSSF/HSSF` 随机访问能力的显式 Rust 映射。
/// 自定义处理器默认使用 [`Self::Unknown`]，由自动选择器保守地按随机访问处理。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WriteHandlerCapability {
    /// 可在严格顺序的常量内存工作表上安全执行。
    StreamingSafe,
    /// 需要保留最近指定数量的行。
    RequiresRowWindow(usize),
    /// 需要任意访问已经写出的行或单元格。
    RequiresRandomAccess,
    /// 需要在 Sheet 全部行完成后执行最终遍历。
    RequiresFinalSheetPass,
    /// 未声明能力；自动模式按需要随机访问处理。
    #[default]
    Unknown,
}

impl WriteHandlerCapability {
    /// 返回该能力是否可直接用于严格常量内存后端。
    #[must_use]
    pub const fn is_streaming_safe(self) -> bool {
        matches!(self, Self::StreamingSafe)
    }
}
