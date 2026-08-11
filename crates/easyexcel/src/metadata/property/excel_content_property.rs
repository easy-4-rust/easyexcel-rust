//! 对应 Java：`com.alibaba.excel.metadata.property.ExcelContentProperty`.
//!
//! See also [`DateTimeFormatProperty`] and [`NumberFormatProperty`] for the
//! annotation-driven format metadata that Java stores on this type.

use super::date_time_format_property::DateTimeFormatProperty;
use super::font_property::FontProperty;
use super::number_format_property::NumberFormatProperty;
use super::style_property::StyleProperty;

/// 对应 Java：`ExcelContentProperty`.
///
/// Java carries a `Field`, `Converter`, [`DateTimeFormatProperty`],
/// [`NumberFormatProperty`], `StyleProperty` and `FontProperty`. Rust keeps
/// the four runtime properties directly; JVM reflection objects are represented
/// by stable field/converter registration keys so the derive and registry layers
/// remain the only reflection owners.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ExcelContentProperty {
    /// Content cell style. (Java `contentStyleProperty`)
    pub content_style_property: Option<StyleProperty>,
    /// Content font style. (Java `contentFontProperty`)
    pub content_font_property: Option<FontProperty>,
    /// Optional date-time format metadata. (Java `dateTimeFormatProperty`)
    pub date_time_format_property: Option<DateTimeFormatProperty>,
    /// Optional number format metadata. (Java `numberFormatProperty`)
    pub number_format_property: Option<NumberFormatProperty>,
    /// Optional date-time format string. (Java `dateTimeFormatProperty.format`)
    pub date_time_format: Option<&'static str>,
    /// Optional number format string. (Java `numberFormatProperty.format`)
    pub number_format: Option<&'static str>,
    /// Rust 后端中立的反射字段标识。对应 Java `field`。
    pub field_name: Option<String>,
    /// Rust 后端中立的 converter 注册键。对应 Java `converter`。
    pub converter_key: Option<String>,
}

impl ExcelContentProperty {
    /// Creates an empty property. (Java `EMPTY = new ExcelContentProperty()`)
    pub const EMPTY: Self = Self {
        content_style_property: None,
        content_font_property: None,
        date_time_format_property: None,
        number_format_property: None,
        date_time_format: None,
        number_format: None,
        field_name: None,
        converter_key: None,
    };

    /// 创建 Java 默认对象。
    #[must_use]
    pub const fn new() -> Self { Self::EMPTY }

    /// Java `getContentStyleProperty`。
    #[must_use]
    pub const fn get_content_style_property(&self) -> Option<&StyleProperty> {
        self.content_style_property.as_ref()
    }
    /// Java `setContentStyleProperty`。
    pub fn set_content_style_property(&mut self, value: Option<StyleProperty>) {
        self.content_style_property = value;
    }
    /// Java `getContentFontProperty`。
    #[must_use]
    pub const fn get_content_font_property(&self) -> Option<&FontProperty> {
        self.content_font_property.as_ref()
    }
    /// Java `setContentFontProperty`。
    pub fn set_content_font_property(&mut self, value: Option<FontProperty>) {
        self.content_font_property = value;
    }
    /// Java `getDateTimeFormatProperty`。
    #[must_use]
    pub const fn get_date_time_format_property(&self) -> Option<&DateTimeFormatProperty> {
        self.date_time_format_property.as_ref()
    }
    /// Java `setDateTimeFormatProperty`。
    pub fn set_date_time_format_property(&mut self, value: Option<DateTimeFormatProperty>) {
        self.date_time_format_property = value;
    }
    /// Java `getNumberFormatProperty`。
    #[must_use]
    pub const fn get_number_format_property(&self) -> Option<&NumberFormatProperty> {
        self.number_format_property.as_ref()
    }
    /// Java `setNumberFormatProperty`。
    pub fn set_number_format_property(&mut self, value: Option<NumberFormatProperty>) {
        self.number_format_property = value;
    }
    /// Java `getField` 的后端中立映射。
    #[must_use]
    pub fn get_field(&self) -> Option<&str> { self.field_name.as_deref() }
    /// Java `setField` 的后端中立映射。
    pub fn set_field(&mut self, value: Option<String>) { self.field_name = value; }
    /// Java `getConverter` 的后端中立映射。
    #[must_use]
    pub fn get_converter(&self) -> Option<&str> { self.converter_key.as_deref() }
    /// Java `setConverter` 的后端中立映射。
    pub fn set_converter(&mut self, value: Option<String>) { self.converter_key = value; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_constant_is_all_none() {
        // 对应 Java：ExcelContentProperty.EMPTY 所有字段为 null
        let empty = ExcelContentProperty::EMPTY;
        assert!(empty.get_content_style_property().is_none());
        assert!(empty.get_content_font_property().is_none());
        assert!(empty.get_date_time_format_property().is_none());
        assert!(empty.get_number_format_property().is_none());
        assert!(empty.get_field().is_none());
        assert!(empty.get_converter().is_none());
    }

    #[test]
    fn new_returns_empty() {
        // 对应 Java：new ExcelContentProperty() 等价于 EMPTY
        let prop = ExcelContentProperty::new();
        assert_eq!(prop, ExcelContentProperty::EMPTY);
    }

    #[test]
    fn default_trait_returns_empty() {
        // 对应 Java：Default 派生等价于 new()
        let prop = ExcelContentProperty::default();
        assert_eq!(prop, ExcelContentProperty::new());
    }

    #[test]
    fn set_and_get_content_style_property() {
        // 对应 Java：contentStyleProperty getter/setter
        let mut prop = ExcelContentProperty::new();
        assert!(prop.get_content_style_property().is_none());
        let style = StyleProperty::new();
        prop.set_content_style_property(Some(style));
        assert!(prop.get_content_style_property().is_some());
        prop.set_content_style_property(None);
        assert!(prop.get_content_style_property().is_none());
    }

    #[test]
    fn set_and_get_content_font_property() {
        // 对应 Java：contentFontProperty getter/setter
        let mut prop = ExcelContentProperty::new();
        assert!(prop.get_content_font_property().is_none());
        let font = FontProperty::new();
        prop.set_content_font_property(Some(font));
        assert!(prop.get_content_font_property().is_some());
        prop.set_content_font_property(None);
        assert!(prop.get_content_font_property().is_none());
    }

    #[test]
    fn set_and_get_date_time_format_property() {
        // 对应 Java：dateTimeFormatProperty getter/setter
        let mut prop = ExcelContentProperty::new();
        assert!(prop.get_date_time_format_property().is_none());
        prop.set_date_time_format_property(None);
        assert!(prop.get_date_time_format_property().is_none());
    }

    #[test]
    fn set_and_get_number_format_property() {
        // 对应 Java：numberFormatProperty getter/setter
        let mut prop = ExcelContentProperty::new();
        assert!(prop.get_number_format_property().is_none());
        prop.set_number_format_property(None);
        assert!(prop.get_number_format_property().is_none());
    }

    #[test]
    fn set_and_get_field() {
        // 对应 Java：field getter/setter
        let mut prop = ExcelContentProperty::new();
        assert!(prop.get_field().is_none());
        prop.set_field(Some("myField".to_owned()));
        assert_eq!(prop.get_field(), Some("myField"));
        prop.set_field(None);
        assert!(prop.get_field().is_none());
    }

    #[test]
    fn set_and_get_converter() {
        // 对应 Java：converter getter/setter
        let mut prop = ExcelContentProperty::new();
        assert!(prop.get_converter().is_none());
        prop.set_converter(Some("MyConverter".to_owned()));
        assert_eq!(prop.get_converter(), Some("MyConverter"));
        prop.set_converter(None);
        assert!(prop.get_converter().is_none());
    }

    #[test]
    fn clone_produces_equal_instance() {
        // 对应 Java：clone 产生相等对象
        let mut prop = ExcelContentProperty::new();
        prop.set_field(Some("f".to_owned()));
        prop.set_converter(Some("c".to_owned()));
        let cloned = prop.clone();
        assert_eq!(prop, cloned);
    }

    #[test]
    fn debug_format_does_not_panic() {
        // 对应 Java：toString 不崩溃
        let prop = ExcelContentProperty::new();
        let _debug = format!("{prop:?}");
    }

    #[test]
    fn hash_consistency() {
        // 对应 Java：相同内容哈希一致
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut a = ExcelContentProperty::new();
        a.set_field(Some("x".to_owned()));
        let mut b = ExcelContentProperty::new();
        b.set_field(Some("x".to_owned()));
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}
