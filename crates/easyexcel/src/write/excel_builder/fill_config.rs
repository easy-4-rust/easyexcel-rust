/// Minimal fill configuration accepted by [`ExcelBuilder::fill`].
///
/// 对应 Java：`com.alibaba.excel.write.metadata.fill.FillConfig` at the
/// builder surface. Stateful template filling remains on
/// `easyexcel_template::FillConfig`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillConfig {
    /// Collection expansion direction. `None` is initialized as vertical.
    /// (Java `FillConfig.direction`)
    pub direction: Option<crate::core::WriteDirection>,
    /// Whether collection fill forces a new row. (Java `FillConfig.forceNewRow`)
    pub force_new_row: bool,
    /// Whether generated cells inherit the template style.
    /// (Java `FillConfig.autoStyle`, default `true`)
    pub auto_style: bool,
    has_init: bool,
}

impl FillConfig {
    /// Creates Java-compatible effective defaults.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn new() -> Self {
        Self {
            direction: None,
            force_new_row: false,
            auto_style: true,
            has_init: false,
        }
    }

    /// Sets the collection expansion direction.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn direction(mut self, direction: crate::core::WriteDirection) -> Self {
        self.direction = Some(direction);
        self
    }

    /// Sets whether collection fill forces a new row.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn force_new_row(mut self, force_new_row: bool) -> Self {
        self.force_new_row = force_new_row;
        self
    }

    /// Sets whether generated cells inherit the template style.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn auto_style(mut self, auto_style: bool) -> Self {
        self.auto_style = auto_style;
        self
    }

    /// Applies Java defaults once. Rust stores effective non-null values, so
    /// initialization only records the lifecycle transition.
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub fn init(&mut self) {
        if !self.has_init {
            self.has_init = true;
        }
    }

    /// Returns whether [`Self::init`] has run.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.fill.FillConfig。
    pub const fn has_init(&self) -> bool {
        self.has_init
    }

    /// Java `getDirection`。
    #[must_use]
    pub const fn get_direction(&self) -> Option<crate::core::WriteDirection> { self.direction }
    /// Java `setDirection`。
    pub const fn set_direction(&mut self, value: Option<crate::core::WriteDirection>) {
        self.direction = value;
    }
    /// Java `getForceNewRow`。
    #[must_use]
    pub const fn get_force_new_row(&self) -> bool { self.force_new_row }
    /// Java `setForceNewRow`。
    pub const fn set_force_new_row(&mut self, value: bool) { self.force_new_row = value; }
    /// Java `getAutoStyle`。
    #[must_use]
    pub const fn get_auto_style(&self) -> bool { self.auto_style }
    /// Java `setAutoStyle`。
    pub const fn set_auto_style(&mut self, value: bool) { self.auto_style = value; }
    /// Java `isHasInit`。
    #[must_use]
    pub const fn is_has_init(&self) -> bool { self.has_init }
    /// Java `setHasInit`。
    pub const fn set_has_init(&mut self, value: bool) { self.has_init = value; }
}

impl Default for FillConfig {
    fn default() -> Self {
        Self::new()
    }
}
