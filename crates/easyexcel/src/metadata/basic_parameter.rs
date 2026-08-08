//! 对应 Java：`com.alibaba.excel.metadata.BasicParameter`.

use crate::CacheLocation;

/// 对应 Java：com.alibaba.excel.metadata.BasicParameter。 Shared read/write builder parameters.
///
/// Java stores a reflective `Class<?> clazz`; Rust stores the type name string
/// because model metadata is resolved at compile time through `ExcelRow`.
///
/// Rust port of Java `BasicParameter`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BasicParameter {
    /// Dynamic header rows. (Java `head`)
    pub head: Option<Vec<Vec<String>>>,
    /// Model type name. (Java `clazz`)
    pub clazz: Option<String>,
    /// Custom converter type names registered on the builder. (Java `customConverterList`)
    pub custom_converter_list: Vec<String>,
    /// Automatic trim for sheet names and cell text. (Java `autoTrim`)
    pub auto_trim: Option<bool>,
    /// Whether Excel 1904 date windowing is enabled. (Java `use1904windowing`)
    pub use1904windowing: Option<bool>,
    /// Locale used for date/number formatting. (Java `locale`)
    pub locale: Option<String>,
    /// Whether scientific notation is used. (Java `useScientificFormat`)
    pub use_scientific_format: Option<bool>,
    /// Field-cache location for reflection metadata. (Java `filedCacheLocation`)
    pub filed_cache_location: Option<CacheLocation>,
}

impl BasicParameter {
    /// 对应 Java：com.alibaba.excel.metadata.BasicParameter。 Creates an empty parameter bag. (Java default constructor)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 对应 Java：com.alibaba.excel.metadata.BasicParameter。 Returns the configured header rows. (Java `getHead()`)
    #[must_use]
    pub fn head(&self) -> Option<&[Vec<String>]> {
        self.head.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.metadata.BasicParameter。 Returns the model type name. (Java `getClazz()`)
    #[must_use]
    pub fn clazz(&self) -> Option<&str> {
        self.clazz.as_deref()
    }

    /// 对应 Java：com.alibaba.excel.metadata.BasicParameter。 Returns custom converter registrations. (Java `getCustomConverterList()`)
    #[must_use]
    pub fn custom_converter_list(&self) -> &[String] {
        &self.custom_converter_list
    }

    /// Java `getHead` 别名。
    #[must_use] pub fn get_head(&self) -> Option<&[Vec<String>]> { self.head.as_deref() }
    /// Java `setHead`。
    pub fn set_head(&mut self, value: Option<Vec<Vec<String>>>) { self.head = value; }
    /// Java `getClazz` 的 Rust 类型名映射。
    #[must_use] pub fn get_clazz(&self) -> Option<&str> { self.clazz.as_deref() }
    /// Java `setClazz` 的 Rust 类型名映射。
    pub fn set_clazz(&mut self, value: Option<String>) { self.clazz = value; }
    /// Java `getCustomConverterList` 别名。
    #[must_use] pub fn get_custom_converter_list(&self) -> &[String] { &self.custom_converter_list }
    /// Java `setCustomConverterList`。
    pub fn set_custom_converter_list(&mut self, value: Vec<String>) { self.custom_converter_list = value; }
    /// Java `getAutoTrim`。
    #[must_use] pub const fn get_auto_trim(&self) -> Option<bool> { self.auto_trim }
    /// Java `setAutoTrim`。
    pub const fn set_auto_trim(&mut self, value: Option<bool>) { self.auto_trim = value; }
    /// Java `getUse1904windowing`。
    #[must_use] pub const fn get_use_1904windowing(&self) -> Option<bool> { self.use1904windowing }
    /// Java `setUse1904windowing`。
    pub const fn set_use_1904windowing(&mut self, value: Option<bool>) { self.use1904windowing = value; }
    /// Java `getLocale` 的语言标签映射。
    #[must_use] pub fn get_locale(&self) -> Option<&str> { self.locale.as_deref() }
    /// Java `setLocale` 的语言标签映射。
    pub fn set_locale(&mut self, value: Option<String>) { self.locale = value; }
    /// Java `getUseScientificFormat`。
    #[must_use] pub const fn get_use_scientific_format(&self) -> Option<bool> { self.use_scientific_format }
    /// Java `setUseScientificFormat`。
    pub const fn set_use_scientific_format(&mut self, value: Option<bool>) { self.use_scientific_format = value; }
    /// Java `getFiledCacheLocation`（保留上游拼写）。
    #[must_use] pub const fn get_filed_cache_location(&self) -> Option<CacheLocation> { self.filed_cache_location }
    /// Java `setFiledCacheLocation`（保留上游拼写）。
    pub const fn set_filed_cache_location(&mut self, value: Option<CacheLocation>) { self.filed_cache_location = value; }
}
