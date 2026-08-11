//! 对应 Java：`com.alibaba.excel.annotation.write.style.ContentFontStyle`。

use crate::{BooleanEnum, ExcelColor, ExcelFontScript, ExcelFontStyle, ExcelUnderline, FontProperty};

/// 内容字体注解的全部参数及 Java 默认值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentFontStyle {
    font_name: &'static str,
    font_height_in_points: i16,
    italic: BooleanEnum,
    strikeout: BooleanEnum,
    color: i16,
    type_offset: i16,
    underline: i8,
    charset: i32,
    bold: BooleanEnum,
}
impl Default for ContentFontStyle {
    fn default() -> Self { Self { font_name: "", font_height_in_points: -1, italic: BooleanEnum::Default, strikeout: BooleanEnum::Default, color: -1, type_offset: -1, underline: -1, charset: -1, bold: BooleanEnum::Default } }
}
impl ContentFontStyle {
    /// 创建 Java 默认参数对象。
    #[must_use] pub fn new() -> Self { Self::default() }
    #[must_use] pub fn font_name(&self) -> &str { &self.font_name }
    pub const fn set_font_name(&mut self, value: &'static str) { self.font_name = value; }
    #[must_use] pub const fn font_height_in_points(&self) -> i16 { self.font_height_in_points }
    pub const fn set_font_height_in_points(&mut self, value: i16) { self.font_height_in_points = value; }
    #[must_use] pub const fn italic(&self) -> BooleanEnum { self.italic }
    pub const fn set_italic(&mut self, value: BooleanEnum) { self.italic = value; }
    #[must_use] pub const fn strikeout(&self) -> BooleanEnum { self.strikeout }
    pub const fn set_strikeout(&mut self, value: BooleanEnum) { self.strikeout = value; }
    #[must_use] pub const fn color(&self) -> i16 { self.color }
    pub const fn set_color(&mut self, value: i16) { self.color = value; }
    #[must_use] pub const fn type_offset(&self) -> i16 { self.type_offset }
    pub const fn set_type_offset(&mut self, value: i16) { self.type_offset = value; }
    #[must_use] pub const fn underline(&self) -> i8 { self.underline }
    pub const fn set_underline(&mut self, value: i8) { self.underline = value; }
    #[must_use] pub const fn charset(&self) -> i32 { self.charset }
    pub const fn set_charset(&mut self, value: i32) { self.charset = value; }
    #[must_use] pub const fn bold(&self) -> BooleanEnum { self.bold }
    pub const fn set_bold(&mut self, value: BooleanEnum) { self.bold = value; }
    /// 转换成已有字体属性；所有 `-1`/`DEFAULT` 值继续保持未指定。
    #[must_use]
    pub fn to_property(&self) -> FontProperty {
        FontProperty::build(ExcelFontStyle {
            font_name: (!self.font_name.is_empty()).then_some(self.font_name),
            font_height_in_points: (self.font_height_in_points >= 0).then_some(f64::from(self.font_height_in_points)),
            italic: self.italic.value(), strikeout: self.strikeout.value(),
            color: u32::try_from(self.color).ok().map(ExcelColor::java_or_rgb),
            type_offset: match self.type_offset { 0 => Some(ExcelFontScript::None), 1 => Some(ExcelFontScript::Superscript), 2 => Some(ExcelFontScript::Subscript), _ => None },
            underline: match self.underline { 0 => Some(ExcelUnderline::None), 1 => Some(ExcelUnderline::Single), 2 => Some(ExcelUnderline::Double), 33 => Some(ExcelUnderline::SingleAccounting), 34 => Some(ExcelUnderline::DoubleAccounting), _ => None },
            charset: u8::try_from(self.charset).ok(), bold: self.bold.value(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_java() {
        let style = ContentFontStyle::new();
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
    fn setters_and_getters() {
        let mut style = ContentFontStyle::new();
        style.set_font_name("Arial");
        assert_eq!(style.font_name(), "Arial");
        style.set_font_height_in_points(12);
        assert_eq!(style.font_height_in_points(), 12);
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
        let style = ContentFontStyle::new();
        let prop = style.to_property();
        // 默认 -1 值产生 None
        assert!(prop.font_name.is_none());
        assert!(prop.font_height_in_points.is_none());
    }

    #[test]
    fn to_property_with_configured_values() {
        let mut style = ContentFontStyle::new();
        style.set_font_name("Calibri");
        style.set_font_height_in_points(11);
        style.set_bold(BooleanEnum::True);
        style.set_italic(BooleanEnum::True);
        style.set_color(0);
        style.set_type_offset(0); // None
        style.set_underline(0); // None
        style.set_charset(1);
        let prop = style.to_property();
        assert_eq!(prop.font_name.as_deref(), Some("Calibri"));
        assert_eq!(prop.font_height_in_points, Some(11.0));
    }

    #[test]
    fn to_property_type_offset_variants() {
        let mut style = ContentFontStyle::new();
        style.set_type_offset(0);
        let prop = style.to_property();
        assert!(prop.type_offset.is_some());
        style.set_type_offset(1);
        let prop = style.to_property();
        assert!(prop.type_offset.is_some());
        style.set_type_offset(2);
        let prop = style.to_property();
        assert!(prop.type_offset.is_some());
        style.set_type_offset(99);
        let prop = style.to_property();
        assert!(prop.type_offset.is_none());
    }

    #[test]
    fn to_property_underline_variants() {
        let mut style = ContentFontStyle::new();
        for val in [0, 1, 2, 33, 34, 99] {
            style.set_underline(val);
            let _prop = style.to_property();
        }
    }

    #[test]
    fn clone_and_eq() {
        let mut a = ContentFontStyle::new();
        a.set_font_name("Arial");
        let b = a;
        assert_eq!(a, b);
    }
}
