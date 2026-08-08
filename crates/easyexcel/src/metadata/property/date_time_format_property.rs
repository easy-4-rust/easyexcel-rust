//! 对应 Java：`com.alibaba.excel.metadata.property.DateTimeFormatProperty`.

/// Date-time format metadata from `@DateTimeFormat`.
///
/// Rust port of Java `DateTimeFormatProperty`.
#[derive(Debug, Clone, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.metadata.property.DateTimeFormatProperty。
pub struct DateTimeFormatProperty {
    /// Format pattern. (Java `format`)
    pub format: String,
    /// Whether 1904 date windowing is enabled. (Java `use1904windowing`)
    pub use1904windowing: bool,
}

impl DateTimeFormatProperty {
    /// 对应 Java：com.alibaba.excel.metadata.property.DateTimeFormatProperty。 Creates a date-time format property. (Java constructor)
    #[must_use]
    pub fn new(format: impl Into<String>, use1904windowing: bool) -> Self {
        Self {
            format: format.into(),
            use1904windowing,
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.property.DateTimeFormatProperty。 Builds from annotation values. (Java `build(DateTimeFormat)`)
    #[must_use]
    pub fn build(format: Option<&str>, use1904windowing: Option<bool>) -> Option<Self> {
        format.map(|format| Self {
            format: format.to_owned(),
            use1904windowing: use1904windowing.unwrap_or(false),
        })
    }

    /// 对应 Java：com.alibaba.excel.metadata.property.DateTimeFormatProperty。 Returns the format pattern. (Java `getFormat()`)
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Returns the 1904-windowing flag. (Java `getUse1904windowing()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.DateTimeFormatProperty。
    pub const fn use1904windowing(&self) -> bool {
        self.use1904windowing
    }

    /// Java `getFormat` 别名。
    #[must_use]
    pub fn get_format(&self) -> &str { &self.format }
    /// Java `setFormat`。
    pub fn set_format(&mut self, value: impl Into<String>) { self.format = value.into(); }
    /// Java `getUse1904windowing` 别名。
    #[must_use]
    pub const fn get_use_1904windowing(&self) -> bool { self.use1904windowing }
    /// Java `setUse1904windowing`。
    pub const fn set_use_1904windowing(&mut self, value: bool) {
        self.use1904windowing = value;
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_build_and_accessors() {
        // 对应 Java：DateTimeFormatProperty 构造、build 与 getter
        let property = DateTimeFormatProperty::new("yyyy-MM-dd", true);
        assert_eq!(property.format(), "yyyy-MM-dd");
        assert!(property.use1904windowing());

        assert!(DateTimeFormatProperty::build(None, Some(true)).is_none());
        let built = DateTimeFormatProperty::build(Some("yyyy/MM/dd"), None).expect("built");
        assert_eq!(built.format(), "yyyy/MM/dd");
        assert!(!built.use1904windowing());

        let built = DateTimeFormatProperty::build(Some("yyyy"), Some(true)).expect("built");
        assert!(built.use1904windowing());
    }
}
