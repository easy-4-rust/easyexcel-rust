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
}

impl Default for FillConfig {
    fn default() -> Self {
        Self {
            direction: FillDirection::Vertical,
            force_new_row: false,
            auto_style: true,
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
        }
    }

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
}
