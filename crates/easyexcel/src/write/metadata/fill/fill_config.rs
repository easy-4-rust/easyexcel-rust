//! 集合填充配置与展开方向。
//!
//! 对应 Java：`com.alibaba.excel.write.metadata.fill.FillConfig`
//!
//! 拆分后仅保留 `FillConfig` 结构体；
//! `FillConfigBuilder` 位于同级 `fill_config/fill_config_builder.rs`。

include!("fill_config/fill_direction.rs");
include!("fill_config/fill_config_builder.rs");

/// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。 Collection fill behavior corresponding to Java `EasyExcel`'s `FillConfig`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillConfig {
    direction: Option<FillDirection>,
    force_new_row: Option<bool>,
    auto_style: Option<bool>,
    has_init: bool,
}

impl Default for FillConfig {
    fn default() -> Self {
        Self {
            direction: None,
            force_new_row: None,
            auto_style: None,
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
            direction: None,
            force_new_row: None,
            auto_style: None,
            has_init: false,
        }
    }

    /// 对应 Java 全参数构造器。
    #[must_use]
    pub const fn with_values(
        direction: Option<FillDirection>,
        force_new_row: Option<bool>,
        auto_style: Option<bool>,
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
        self.direction = Some(direction);
        self
    }

    /// Controls whether rows below a vertical template row are shifted.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn force_new_row(mut self, force_new_row: bool) -> Self {
        self.force_new_row = Some(force_new_row);
        self
    }

    /// Controls whether cloned cells retain the template cell style.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn auto_style(mut self, auto_style: bool) -> Self {
        self.auto_style = Some(auto_style);
        self
    }

    /// Returns the configured expansion direction.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn get_direction(self) -> Option<FillDirection> {
        self.direction
    }

    /// Returns whether vertical filling shifts following rows.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn get_force_new_row(self) -> Option<bool> {
        self.force_new_row
    }

    /// Returns whether template style is inherited.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn get_auto_style(self) -> Option<bool> {
        self.auto_style
    }

    /// Java `setDirection`。
    pub const fn set_direction(&mut self, value: Option<FillDirection>) { self.direction = value; }
    /// Java `setForceNewRow`。
    pub const fn set_force_new_row(&mut self, value: Option<bool>) { self.force_new_row = value; }
    /// Java `setAutoStyle`。
    pub const fn set_auto_style(&mut self, value: Option<bool>) { self.auto_style = value; }
    /// Java `init`，仅执行一次并物化三个 nullable 字段的默认值。
    pub fn init(&mut self) {
        if self.has_init {
            return;
        }
        self.direction.get_or_insert(FillDirection::Vertical);
        self.force_new_row.get_or_insert(false);
        self.auto_style.get_or_insert(true);
        self.has_init = true;
    }
    /// Java `isHasInit`。
    #[must_use]
    pub const fn is_has_init(self) -> bool { self.has_init }
    /// Java `setHasInit`。
    pub const fn set_has_init(&mut self, value: bool) { self.has_init = value; }
    /// 返回执行 `init` 后的有效方向，不改变当前对象。
    #[must_use]
    pub fn effective_direction(self) -> FillDirection {
        self.direction.unwrap_or(FillDirection::Vertical)
    }
    /// 返回执行 `init` 后的有效强制新行配置，不改变当前对象。
    #[must_use]
    pub fn effective_force_new_row(self) -> bool { self.force_new_row.unwrap_or(false) }
    /// 返回执行 `init` 后的有效自动样式配置，不改变当前对象。
    #[must_use]
    pub fn effective_auto_style(self) -> bool { self.auto_style.unwrap_or(true) }
}
