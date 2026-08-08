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
