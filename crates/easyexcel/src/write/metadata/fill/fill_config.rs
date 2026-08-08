//! 集合填充配置与展开方向。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.fill.FillConfig`

include!("fill_config/fill_direction.rs");

/// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。 Collection fill behavior corresponding to Java `EasyExcel`'s `FillConfig`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillConfig {
    direction: FillDirection,
    force_new_row: bool,
    auto_style: bool,
    has_init: bool,
}

impl Default for FillConfig {
    fn default() -> Self {
        Self {
            direction: FillDirection::Vertical,
            force_new_row: false,
            auto_style: true,
            has_init: false,
        }
    }
}

impl FillConfig {
    /// Creates Java-compatible default fill configuration.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn new() -> Self {
        Self {
            direction: FillDirection::Vertical,
            force_new_row: false,
            auto_style: true,
            has_init: false,
        }
    }

    /// 对应 Java 全参数构造器。
    #[must_use]
    pub const fn with_values(
        direction: FillDirection,
        force_new_row: bool,
        auto_style: bool,
        has_init: bool,
    ) -> Self {
        Self { direction, force_new_row, auto_style, has_init }
    }

    /// 对应 Java `FillConfig.builder()`。
    #[must_use]
    pub const fn builder() -> FillConfigBuilder { FillConfigBuilder::new() }

    /// Sets vertical or horizontal collection expansion.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn direction(mut self, direction: FillDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Controls whether rows below a vertical template row are shifted.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn force_new_row(mut self, force_new_row: bool) -> Self {
        self.force_new_row = force_new_row;
        self
    }

    /// Controls whether cloned cells retain the template cell style.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn auto_style(mut self, auto_style: bool) -> Self {
        self.auto_style = auto_style;
        self
    }

    /// Returns the configured expansion direction.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn get_direction(self) -> FillDirection {
        self.direction
    }

    /// Returns whether vertical filling shifts following rows.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn get_force_new_row(self) -> bool {
        self.force_new_row
    }

    /// Returns whether template style is inherited.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn get_auto_style(self) -> bool {
        self.auto_style
    }

    /// Java `setDirection`。
    pub const fn set_direction(&mut self, value: FillDirection) { self.direction = value; }
    /// Java `setForceNewRow`。
    pub const fn set_force_new_row(&mut self, value: bool) { self.force_new_row = value; }
    /// Java `setAutoStyle`。
    pub const fn set_auto_style(&mut self, value: bool) { self.auto_style = value; }
    /// Java `init`，仅记录一次初始化并保持已经显式设置的有效值。
    pub const fn init(&mut self) { self.has_init = true; }
    /// Java `isHasInit`。
    #[must_use]
    pub const fn is_has_init(self) -> bool { self.has_init }
    /// Java `setHasInit`。
    pub const fn set_has_init(&mut self, value: bool) { self.has_init = value; }
}

/// 对应 Java：`FillConfig.FillConfigBuilder`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillConfigBuilder {
    value: FillConfig,
}

impl FillConfigBuilder {
    /// 创建 builder。
    #[must_use]
    pub const fn new() -> Self { Self { value: FillConfig::new() } }
    /// 设置方向。
    #[must_use]
    pub const fn direction(mut self, value: FillDirection) -> Self {
        self.value.direction = value;
        self
    }
    /// 设置强制新行。
    #[must_use]
    pub const fn force_new_row(mut self, value: bool) -> Self {
        self.value.force_new_row = value;
        self
    }
    /// 设置自动样式。
    #[must_use]
    pub const fn auto_style(mut self, value: bool) -> Self {
        self.value.auto_style = value;
        self
    }
    /// 构建配置。
    #[must_use]
    pub const fn build(self) -> FillConfig { self.value }
}

impl Default for FillConfigBuilder {
    fn default() -> Self { Self::new() }
}
