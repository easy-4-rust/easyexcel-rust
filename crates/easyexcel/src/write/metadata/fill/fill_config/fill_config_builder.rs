// 集合填充配置构建器。
// 对应 Java：`com.alibaba.excel.write.metadata.fill.FillConfig.FillConfigBuilder`。
// 从 `fill_config.rs` 拆分而来，遵循"一个 .rs 文件只对应一个 Java 对象"规范。

/// 对应 Java：`FillConfig.FillConfigBuilder`。
///
/// 提供与 Java Lombok `@Builder` 一致的流式构建接口。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillConfigBuilder {
    value: FillConfig,
}

impl FillConfigBuilder {
    /// 创建 builder。
    #[must_use]
    pub const fn new() -> Self { Self { value: FillConfig::new() } }
    /// 设置方向。
    ///
    /// # 参数
    /// - `value`: 集合展开方向。
    #[must_use]
    pub const fn direction(mut self, value: FillDirection) -> Self {
        self.value.direction = Some(value);
        self
    }
    /// 设置强制新行。
    ///
    /// # 参数
    /// - `value`: 是否强制新行。
    #[must_use]
    pub const fn force_new_row(mut self, value: bool) -> Self {
        self.value.force_new_row = Some(value);
        self
    }
    /// 设置自动样式。
    ///
    /// # 参数
    /// - `value`: 是否保留模板样式。
    #[must_use]
    pub const fn auto_style(mut self, value: bool) -> Self {
        self.value.auto_style = Some(value);
        self
    }
    /// 设置内部初始化标志。对应 Lombok builder 的 `hasInit(boolean)`。
    ///
    /// # 参数
    /// - `value`: 初始化标志值。
    #[must_use]
    pub const fn has_init(mut self, value: bool) -> Self {
        self.value.has_init = value;
        self
    }
    /// 构建配置。
    ///
    /// # 返回
    /// 构建完成的 `FillConfig` 实例。
    #[must_use]
    pub const fn build(self) -> FillConfig { self.value }
}

impl Default for FillConfigBuilder {
    fn default() -> Self { Self::new() }
}
