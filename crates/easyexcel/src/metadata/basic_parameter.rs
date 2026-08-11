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
    #[must_use]
    pub fn get_head(&self) -> Option<&[Vec<String>]> {
        self.head.as_deref()
    }
    /// Java `setHead`。
    pub fn set_head(&mut self, value: Option<Vec<Vec<String>>>) {
        self.head = value;
    }
    /// Java `getClazz` 的 Rust 类型名映射。
    #[must_use]
    pub fn get_clazz(&self) -> Option<&str> {
        self.clazz.as_deref()
    }
    /// Java `setClazz` 的 Rust 类型名映射。
    pub fn set_clazz(&mut self, value: Option<String>) {
        self.clazz = value;
    }
    /// Java `getCustomConverterList` 别名。
    #[must_use]
    pub fn get_custom_converter_list(&self) -> &[String] {
        &self.custom_converter_list
    }
    /// Java `setCustomConverterList`。
    pub fn set_custom_converter_list(&mut self, value: Vec<String>) {
        self.custom_converter_list = value;
    }
    /// Java `getAutoTrim`。
    #[must_use]
    pub const fn get_auto_trim(&self) -> Option<bool> {
        self.auto_trim
    }
    /// Java `setAutoTrim`。
    pub const fn set_auto_trim(&mut self, value: Option<bool>) {
        self.auto_trim = value;
    }
    /// Java `getUse1904windowing`。
    #[must_use]
    pub const fn get_use_1904windowing(&self) -> Option<bool> {
        self.use1904windowing
    }
    /// Java `getUse1904windowing` 的逐字符 snake_case 名称。
    #[must_use]
    pub const fn get_use1904windowing(&self) -> Option<bool> {
        self.use1904windowing
    }
    /// Java `setUse1904windowing`。
    pub const fn set_use_1904windowing(&mut self, value: Option<bool>) {
        self.use1904windowing = value;
    }
    /// Java `setUse1904windowing` 的逐字符 snake_case 名称。
    pub const fn set_use1904windowing(&mut self, value: Option<bool>) {
        self.use1904windowing = value;
    }
    /// Java `getLocale` 的语言标签映射。
    #[must_use]
    pub fn get_locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }
    /// Java `setLocale` 的语言标签映射。
    pub fn set_locale(&mut self, value: Option<String>) {
        self.locale = value;
    }
    /// Java `getUseScientificFormat`。
    #[must_use]
    pub const fn get_use_scientific_format(&self) -> Option<bool> {
        self.use_scientific_format
    }
    /// Java `setUseScientificFormat`。
    pub const fn set_use_scientific_format(&mut self, value: Option<bool>) {
        self.use_scientific_format = value;
    }
    /// Java `getFiledCacheLocation`（保留上游拼写）。
    #[must_use]
    pub const fn get_filed_cache_location(&self) -> Option<CacheLocation> {
        self.filed_cache_location
    }
    /// Java `setFiledCacheLocation`（保留上游拼写）。
    pub const fn set_filed_cache_location(&mut self, value: Option<CacheLocation>) {
        self.filed_cache_location = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_default() {
        // 对应 Java：BasicParameter 无参构造器
        let param = BasicParameter::new();
        assert_eq!(param, BasicParameter::default());
    }

    #[test]
    fn default_all_fields_none_or_empty() {
        // 对应 Java：Default 所有字段为 null/空
        let param = BasicParameter::default();
        assert!(param.head().is_none());
        assert!(param.clazz().is_none());
        assert!(param.custom_converter_list().is_empty());
        assert!(param.get_auto_trim().is_none());
        assert!(param.get_use_1904windowing().is_none());
        assert!(param.get_locale().is_none());
        assert!(param.get_use_scientific_format().is_none());
        assert!(param.get_filed_cache_location().is_none());
    }

    #[test]
    fn head_setter_and_getter() {
        // 对应 Java：head getter/setter
        let mut param = BasicParameter::new();
        assert!(param.get_head().is_none());
        let head = vec![vec!["Name".to_owned(), "Age".to_owned()]];
        param.set_head(Some(head));
        assert_eq!(param.get_head().unwrap().len(), 1);
        param.set_head(None);
        assert!(param.get_head().is_none());
    }

    #[test]
    fn clazz_setter_and_getter() {
        // 对应 Java：clazz getter/setter
        let mut param = BasicParameter::new();
        assert!(param.get_clazz().is_none());
        param.set_clazz(Some("MyModel".to_owned()));
        assert_eq!(param.get_clazz(), Some("MyModel"));
        param.set_clazz(None);
        assert!(param.get_clazz().is_none());
    }

    #[test]
    fn custom_converter_list_setter_and_getter() {
        // 对应 Java：customConverterList getter/setter
        let mut param = BasicParameter::new();
        assert!(param.get_custom_converter_list().is_empty());
        param.set_custom_converter_list(vec!["C1".to_owned(), "C2".to_owned()]);
        assert_eq!(param.get_custom_converter_list().len(), 2);
    }

    #[test]
    fn auto_trim_setter_and_getter() {
        // 对应 Java：autoTrim getter/setter
        let mut param = BasicParameter::new();
        param.set_auto_trim(Some(true));
        assert_eq!(param.get_auto_trim(), Some(true));
    }

    #[test]
    fn use_1904windowing_setter_and_getter() {
        // 对应 Java：use1904windowing getter/setter（两种命名风格）
        let mut param = BasicParameter::new();
        param.set_use_1904windowing(Some(true));
        assert_eq!(param.get_use_1904windowing(), Some(true));
        assert_eq!(param.get_use1904windowing(), Some(true));
        param.set_use1904windowing(Some(false));
        assert_eq!(param.get_use_1904windowing(), Some(false));
    }

    #[test]
    fn locale_setter_and_getter() {
        // 对应 Java：locale getter/setter
        let mut param = BasicParameter::new();
        param.set_locale(Some("zh_CN".to_owned()));
        assert_eq!(param.get_locale(), Some("zh_CN"));
        param.set_locale(None);
        assert!(param.get_locale().is_none());
    }

    #[test]
    fn use_scientific_format_setter_and_getter() {
        // 对应 Java：useScientificFormat getter/setter
        let mut param = BasicParameter::new();
        param.set_use_scientific_format(Some(true));
        assert_eq!(param.get_use_scientific_format(), Some(true));
    }

    #[test]
    fn filed_cache_location_setter_and_getter() {
        // 对应 Java：filedCacheLocation getter/setter
        let mut param = BasicParameter::new();
        param.set_filed_cache_location(Some(CacheLocation::ThreadLocal));
        assert_eq!(
            param.get_filed_cache_location(),
            Some(CacheLocation::ThreadLocal)
        );
        param.set_filed_cache_location(None);
        assert!(param.get_filed_cache_location().is_none());
    }

    #[test]
    fn clone_produces_equal_instance() {
        // 对应 Java：clone
        let mut param = BasicParameter::new();
        param.set_clazz(Some("C".to_owned()));
        param.set_auto_trim(Some(true));
        let cloned = param.clone();
        assert_eq!(param, cloned);
    }

    #[test]
    fn debug_format_does_not_panic() {
        // 对应 Java：toString
        let param = BasicParameter::new();
        let _debug = format!("{param:?}");
    }

    #[test]
    fn head_rust_style_accessor() {
        // 对应 Java：head() Rust 风格访问器
        let mut param = BasicParameter::new();
        assert!(param.head().is_none());
        param.set_head(Some(vec![vec!["A".to_owned()]]));
        assert!(param.head().is_some());
    }

    #[test]
    fn clazz_rust_style_accessor() {
        // 对应 Java：clazz() Rust 风格访问器
        let mut param = BasicParameter::new();
        assert!(param.clazz().is_none());
        param.set_clazz(Some("Test".to_owned()));
        assert_eq!(param.clazz(), Some("Test"));
    }

    #[test]
    fn custom_converter_list_rust_style_accessor() {
        // 对应 Java：custom_converter_list() Rust 风格访问器
        let param = BasicParameter::new();
        assert!(param.custom_converter_list().is_empty());
    }
}
