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
