/// Minimal fill configuration at the [`ExcelBuilder`](crate::WriteContext) surface.
///
/// 对应 Java：`com.alibaba.excel.write.metadata.fill.FillConfig` fields used by
/// `ExcelBuilderImpl.fill`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WriteFillConfig {
    /// Whether collection fill forces a new row. (Java `FillConfig.forceNewRow`)
    pub force_new_row: bool,
    /// Collection expansion direction when supplied by the caller.
    pub direction: Option<WriteDirection>,
    /// Whether newly created cells inherit the template style.
    /// (Java `FillConfig.autoStyle`, default `true`)
    pub auto_style: bool,
}

impl WriteFillConfig {
    /// Creates Java-compatible defaults (`vertical`, no forced row, auto style).
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn new() -> Self {
        Self {
            force_new_row: false,
            direction: None,
            auto_style: true,
        }
    }
}

impl Default for WriteFillConfig {
    fn default() -> Self {
        Self::new()
    }
}

