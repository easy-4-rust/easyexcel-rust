//! 对应 Java：`com.alibaba.excel.metadata.GlobalConfiguration`.

use crate::CacheLocation;

/// 对应 Java：com.alibaba.excel.metadata.GlobalConfiguration。 Global read/write configuration carried by holders.
///
/// Rust port of Java `GlobalConfiguration`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalConfiguration {
    /// Automatic trim for sheet names and cell text. (Java `autoTrim`)
    pub auto_trim: bool,
    /// Whether Excel 1904 date windowing is enabled. (Java `use1904windowing`)
    pub use1904windowing: bool,
    /// Locale used for date/number formatting. (Java `locale`)
    pub locale: String,
    /// Whether scientific notation is used. (Java `useScientificFormat`)
    pub use_scientific_format: bool,
    /// Field-cache location for reflection metadata. (Java `filedCacheLocation`)
    pub filed_cache_location: CacheLocation,
}

impl Default for GlobalConfiguration {
    /// 对应 Java： default constructor values.
    fn default() -> Self {
        Self {
            auto_trim: true,
            use1904windowing: false,
            locale: "default".to_owned(),
            use_scientific_format: false,
            filed_cache_location: CacheLocation::ThreadLocal,
        }
    }
}

impl GlobalConfiguration {
    /// 对应 Java：com.alibaba.excel.metadata.GlobalConfiguration。 Creates a global configuration with Java default values. (Java constructor)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the auto-trim flag. (Java `getAutoTrim()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.GlobalConfiguration。
    pub const fn auto_trim(&self) -> bool {
        self.auto_trim
    }

    /// Returns the 1904-windowing flag. (Java `getUse1904windowing()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.GlobalConfiguration。
    pub const fn use1904windowing(&self) -> bool {
        self.use1904windowing
    }

    /// 对应 Java：com.alibaba.excel.metadata.GlobalConfiguration。 Returns the locale name. (Java `getLocale()`)
    #[must_use]
    pub fn locale(&self) -> &str {
        &self.locale
    }

    /// Returns the scientific-format flag. (Java `getUseScientificFormat()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.GlobalConfiguration。
    pub const fn use_scientific_format(&self) -> bool {
        self.use_scientific_format
    }

    /// Returns the field-cache location. (Java `getFiledCacheLocation()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.GlobalConfiguration。
    pub const fn filed_cache_location(&self) -> CacheLocation {
        self.filed_cache_location
    }

    /// Java `getAutoTrim` 别名。
    #[must_use]
    pub const fn get_auto_trim(&self) -> bool { self.auto_trim }
    /// Java `setAutoTrim`。
    pub const fn set_auto_trim(&mut self, value: bool) { self.auto_trim = value; }
    /// Java `getUse1904windowing` 别名。
    #[must_use]
    pub const fn get_use_1904windowing(&self) -> bool { self.use1904windowing }
    /// Java `setUse1904windowing`。
    pub const fn set_use_1904windowing(&mut self, value: bool) {
        self.use1904windowing = value;
    }
    /// Java `getLocale` 别名。
    #[must_use]
    pub fn get_locale(&self) -> &str { &self.locale }
    /// Java `setLocale`。
    pub fn set_locale(&mut self, value: impl Into<String>) { self.locale = value.into(); }
    /// Java `getUseScientificFormat` 别名。
    #[must_use]
    pub const fn get_use_scientific_format(&self) -> bool { self.use_scientific_format }
    /// Java `setUseScientificFormat`。
    pub const fn set_use_scientific_format(&mut self, value: bool) {
        self.use_scientific_format = value;
    }
    /// Java `getFiledCacheLocation` 别名（保留原拼写）。
    #[must_use]
    pub const fn get_filed_cache_location(&self) -> CacheLocation {
        self.filed_cache_location
    }
    /// Java `setFiledCacheLocation`（保留原拼写）。
    pub const fn set_filed_cache_location(&mut self, value: CacheLocation) {
        self.filed_cache_location = value;
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn use_scientific_format_accessor() {
        // 对应 Java：GlobalConfiguration.getUseScientificFormat
        let config = GlobalConfiguration::new();
        assert!(!config.use_scientific_format());
        let mut config = config;
        config.use_scientific_format = true;
        assert!(config.use_scientific_format());
    }
}
