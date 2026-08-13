//! 对应 Java：`com.alibaba.excel.annotation.write.style.HeadFontStyle`。

use super::ContentFontStyle;
use crate::{BooleanEnum, FontProperty};

/// 表头字体注解；字段和默认值与 `ContentFontStyle` 相同，但保持独立 Java 类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeadFontStyle {
    values: ContentFontStyle,
}
impl HeadFontStyle {
    /// 创建 Java 默认参数对象。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub fn font_name(&self) -> &str {
        self.values.font_name()
    }
    pub const fn set_font_name(&mut self, value: &'static str) {
        self.values.set_font_name(value);
    }
    #[must_use]
    pub const fn font_height_in_points(&self) -> i16 {
        self.values.font_height_in_points()
    }
    pub const fn set_font_height_in_points(&mut self, value: i16) {
        self.values.set_font_height_in_points(value);
    }
    #[must_use]
    pub const fn italic(&self) -> BooleanEnum {
        self.values.italic()
    }
    pub const fn set_italic(&mut self, value: BooleanEnum) {
        self.values.set_italic(value);
    }
    #[must_use]
    pub const fn strikeout(&self) -> BooleanEnum {
        self.values.strikeout()
    }
    pub const fn set_strikeout(&mut self, value: BooleanEnum) {
        self.values.set_strikeout(value);
    }
    #[must_use]
    pub const fn color(&self) -> i16 {
        self.values.color()
    }
    pub const fn set_color(&mut self, value: i16) {
        self.values.set_color(value);
    }
    #[must_use]
    pub const fn type_offset(&self) -> i16 {
        self.values.type_offset()
    }
    pub const fn set_type_offset(&mut self, value: i16) {
        self.values.set_type_offset(value);
    }
    #[must_use]
    pub const fn underline(&self) -> i8 {
        self.values.underline()
    }
    pub const fn set_underline(&mut self, value: i8) {
        self.values.set_underline(value);
    }
    #[must_use]
    pub const fn charset(&self) -> i32 {
        self.values.charset()
    }
    pub const fn set_charset(&mut self, value: i32) {
        self.values.set_charset(value);
    }
    #[must_use]
    pub const fn bold(&self) -> BooleanEnum {
        self.values.bold()
    }
    pub const fn set_bold(&mut self, value: BooleanEnum) {
        self.values.set_bold(value);
    }
    /// 转换为运行期字体属性。
    #[must_use]
    pub fn to_property(&self) -> FontProperty {
        self.values.to_property()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_returns_default() {
        let style = HeadFontStyle::new();
        assert_eq!(style, HeadFontStyle::default());
    }

    #[test]
    fn default_values_match_java_sentinel() {
        let style = HeadFontStyle::default();
        assert!(style.font_name().is_empty());
        assert_eq!(style.font_height_in_points(), -1);
        assert_eq!(style.italic(), BooleanEnum::Default);
        assert_eq!(style.strikeout(), BooleanEnum::Default);
        assert_eq!(style.color(), -1);
        assert_eq!(style.type_offset(), -1);
        assert_eq!(style.underline(), -1);
        assert_eq!(style.charset(), -1);
        assert_eq!(style.bold(), BooleanEnum::Default);
    }

    #[test]
    fn setters_and_getters_roundtrip() {
        let mut style = HeadFontStyle::new();
        style.set_font_name("Arial");
        assert_eq!(style.font_name(), "Arial");
        style.set_font_height_in_points(14);
        assert_eq!(style.font_height_in_points(), 14);
        style.set_italic(BooleanEnum::True);
        assert_eq!(style.italic(), BooleanEnum::True);
        style.set_strikeout(BooleanEnum::True);
        assert_eq!(style.strikeout(), BooleanEnum::True);
        style.set_color(255);
        assert_eq!(style.color(), 255);
        style.set_type_offset(1);
        assert_eq!(style.type_offset(), 1);
        style.set_underline(2);
        assert_eq!(style.underline(), 2);
        style.set_charset(134);
        assert_eq!(style.charset(), 134);
        style.set_bold(BooleanEnum::True);
        assert_eq!(style.bold(), BooleanEnum::True);
    }

    #[test]
    fn to_property_with_all_defaults() {
        let style = HeadFontStyle::default();
        let prop = style.to_property();
        // Default -1 values produce None in FontProperty
        assert!(prop.font_name.is_none());
        assert!(prop.font_height_in_points.is_none());
        assert!(prop.italic.is_none());
        assert!(prop.strikeout.is_none());
        assert!(prop.color.is_none());
        assert!(prop.type_offset.is_none());
        assert!(prop.underline.is_none());
        assert!(prop.charset.is_none());
        assert!(prop.bold.is_none());
    }

    #[test]
    fn to_property_with_configured_values() {
        let mut style = HeadFontStyle::new();
        style.set_font_name("Calibri");
        style.set_font_height_in_points(11);
        style.set_bold(BooleanEnum::True);
        style.set_italic(BooleanEnum::True);
        style.set_color(0);
        style.set_type_offset(0);
        style.set_underline(0);
        style.set_charset(1);
        let prop = style.to_property();
        assert_eq!(prop.font_name.as_deref(), Some("Calibri"));
        assert_eq!(prop.font_height_in_points, Some(11.0));
        assert_eq!(prop.bold, Some(true));
        assert_eq!(prop.italic, Some(true));
        assert!(prop.type_offset.is_some());
        assert!(prop.underline.is_some());
    }

    #[test]
    fn to_property_type_offset_variants() {
        let mut style = HeadFontStyle::new();
        style.set_type_offset(0);
        assert!(style.to_property().type_offset.is_some());
        style.set_type_offset(1);
        assert!(style.to_property().type_offset.is_some());
        style.set_type_offset(2);
        assert!(style.to_property().type_offset.is_some());
        style.set_type_offset(99);
        assert!(style.to_property().type_offset.is_none());
    }

    #[test]
    fn to_property_underline_variants() {
        let mut style = HeadFontStyle::new();
        for val in [0, 1, 2, 33, 34, 99] {
            style.set_underline(val);
            let _prop = style.to_property();
        }
    }

    #[test]
    fn copy_clone_eq() {
        let mut a = HeadFontStyle::new();
        a.set_font_name("Arial");
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_contains_struct_name() {
        let style = HeadFontStyle::new();
        let text = format!("{style:?}");
        assert!(text.contains("HeadFontStyle"));
    }
}
