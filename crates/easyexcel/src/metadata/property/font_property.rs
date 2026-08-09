//! 对应 Java：`com.alibaba.excel.metadata.property.FontProperty`。

use std::hash::{Hash, Hasher};

use crate::core::{ExcelColor, ExcelFontScript, ExcelFontStyle, ExcelUnderline, WriteFont};

/// 注解解析后的运行期字体属性。
///
/// 对应 Java：`com.alibaba.excel.metadata.property.FontProperty`。注解输入仍由
/// 可复制的 [`ExcelFontStyle`] 承载；进入运行期后字体名称改为拥有所有权的
/// `String`，因此不会把用户动态设置的名称错误收窄成 `&'static str`。
#[derive(Debug, Clone, Default)]
pub struct FontProperty {
    /// 字体名称。对应 Java `fontName`。
    pub font_name: Option<String>,
    /// 字号（point）。对应 Java `fontHeightInPoints`。
    pub font_height_in_points: Option<f64>,
    /// 是否斜体。
    pub italic: Option<bool>,
    /// 是否删除线。
    pub strikeout: Option<bool>,
    /// 字体颜色。
    pub color: Option<ExcelColor>,
    /// 上下标类型。
    pub type_offset: Option<ExcelFontScript>,
    /// 下划线类型。
    pub underline: Option<ExcelUnderline>,
    /// 字符集。
    pub charset: Option<u8>,
    /// 是否粗体。
    pub bold: Option<bool>,
}

impl PartialEq for FontProperty {
    fn eq(&self, other: &Self) -> bool {
        self.font_name == other.font_name
            && self.font_height_in_points.map(java_double_bits)
                == other.font_height_in_points.map(java_double_bits)
            && self.italic == other.italic
            && self.strikeout == other.strikeout
            && self.color == other.color
            && self.type_offset == other.type_offset
            && self.underline == other.underline
            && self.charset == other.charset
            && self.bold == other.bold
    }
}

impl Eq for FontProperty {}

impl Hash for FontProperty {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_name.hash(state);
        self.font_height_in_points.map(java_double_bits).hash(state);
        self.italic.hash(state);
        self.strikeout.hash(state);
        self.color.hash(state);
        self.type_offset.hash(state);
        self.underline.hash(state);
        self.charset.hash(state);
        self.bold.hash(state);
    }
}

fn java_double_bits(value: f64) -> u64 {
    if value.is_nan() { f64::NAN.to_bits() } else { value.to_bits() }
}

impl FontProperty {
    /// 创建所有字段均保持 Java `null` 语义的属性对象。
    #[must_use]
    pub const fn new() -> Self {
        Self {
            font_name: None,
            font_height_in_points: None,
            italic: None,
            strikeout: None,
            color: None,
            type_offset: None,
            underline: None,
            charset: None,
            bold: None,
        }
    }

    /// 从派生宏或注解等价字体样式构建运行期属性。
    ///
    /// 对应 Java：`FontProperty.build(HeadFontStyle|ContentFontStyle)`。
    #[must_use]
    pub fn build(style: ExcelFontStyle) -> Self {
        Self {
            font_name: style.font_name.map(str::to_owned),
            font_height_in_points: style.font_height_in_points,
            italic: style.italic,
            strikeout: style.strikeout,
            color: style.color,
            type_offset: style.type_offset,
            underline: style.underline,
            charset: style.charset,
            bold: style.bold,
        }
    }

    /// 转换为 Java 公共运行期字体对象，不丢失动态字体名称。
    #[must_use]
    pub fn to_write_font(&self) -> WriteFont {
        let mut font = WriteFont::new();
        if let Some(value) = &self.font_name {
            font = font.font_name(value.clone());
        }
        if let Some(value) = self.font_height_in_points {
            font = font.font_height_in_points(value);
        }
        if let Some(value) = self.italic {
            font = font.italic(value);
        }
        if let Some(value) = self.strikeout {
            font = font.strikeout(value);
        }
        if let Some(value) = self.color {
            font = font.color(value);
        }
        if let Some(value) = self.type_offset {
            font = font.type_offset(value);
        }
        if let Some(value) = self.underline {
            font = font.underline(value);
        }
        if let Some(value) = self.charset {
            font = font.charset(value);
        }
        if let Some(value) = self.bold {
            font = font.bold(value);
        }
        font
    }

    /// 返回字体名称。
    #[must_use]
    pub fn font_name(&self) -> Option<&str> { self.font_name.as_deref() }
    /// Java `getFontName` 别名。
    #[must_use]
    pub fn get_font_name(&self) -> Option<&str> { self.font_name() }
    /// 设置或清空运行期字体名称。
    pub fn set_font_name(&mut self, value: Option<String>) { self.font_name = value; }
    /// 返回字号（point）。
    #[must_use]
    pub const fn font_height_in_points(&self) -> Option<f64> { self.font_height_in_points }
    /// Java `getFontHeightInPoints` 别名。
    #[must_use]
    pub const fn get_font_height_in_points(&self) -> Option<f64> { self.font_height_in_points }
    /// 设置字号（point）。
    pub const fn set_font_height_in_points(&mut self, value: Option<f64>) { self.font_height_in_points = value; }
    /// 返回斜体标志。
    #[must_use]
    pub const fn italic(&self) -> Option<bool> { self.italic }
    /// Java `getItalic` 别名。
    #[must_use]
    pub const fn get_italic(&self) -> Option<bool> { self.italic() }
    /// 设置斜体标志。
    pub const fn set_italic(&mut self, value: Option<bool>) { self.italic = value; }
    /// 返回删除线标志。
    #[must_use]
    pub const fn strikeout(&self) -> Option<bool> { self.strikeout }
    /// Java `getStrikeout` 别名。
    #[must_use]
    pub const fn get_strikeout(&self) -> Option<bool> { self.strikeout() }
    /// 设置删除线标志。
    pub const fn set_strikeout(&mut self, value: Option<bool>) { self.strikeout = value; }
    /// 返回字体颜色。
    #[must_use]
    pub const fn color(&self) -> Option<ExcelColor> { self.color }
    /// Java `getColor` 别名。
    #[must_use]
    pub const fn get_color(&self) -> Option<ExcelColor> { self.color() }
    /// 设置字体颜色。
    pub const fn set_color(&mut self, value: Option<ExcelColor>) { self.color = value; }
    /// 返回上下标类型。
    #[must_use]
    pub const fn type_offset(&self) -> Option<ExcelFontScript> { self.type_offset }
    /// Java `getTypeOffset` 别名。
    #[must_use]
    pub const fn get_type_offset(&self) -> Option<ExcelFontScript> { self.type_offset() }
    /// 设置上下标类型。
    pub const fn set_type_offset(&mut self, value: Option<ExcelFontScript>) { self.type_offset = value; }
    /// 返回下划线类型。
    #[must_use]
    pub const fn underline(&self) -> Option<ExcelUnderline> { self.underline }
    /// Java `getUnderline` 别名。
    #[must_use]
    pub const fn get_underline(&self) -> Option<ExcelUnderline> { self.underline() }
    /// 设置下划线类型。
    pub const fn set_underline(&mut self, value: Option<ExcelUnderline>) { self.underline = value; }
    /// 返回字符集。
    #[must_use]
    pub const fn charset(&self) -> Option<u8> { self.charset }
    /// Java `getCharset` 别名。
    #[must_use]
    pub const fn get_charset(&self) -> Option<u8> { self.charset() }
    /// 设置字符集。
    pub const fn set_charset(&mut self, value: Option<u8>) { self.charset = value; }
    /// 返回粗体标志。
    #[must_use]
    pub const fn bold(&self) -> Option<bool> { self.bold }
    /// Java `getBold` 别名。
    #[must_use]
    pub const fn get_bold(&self) -> Option<bool> { self.bold() }
    /// 设置粗体标志。
    pub const fn set_bold(&mut self, value: Option<bool>) { self.bold = value; }
}
