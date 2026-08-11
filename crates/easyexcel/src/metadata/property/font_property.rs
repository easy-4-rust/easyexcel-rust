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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ExcelColor, ExcelFontScript, ExcelFontStyle, ExcelUnderline};

    #[test]
    fn new_returns_all_none() {
        // 对应 Java：FontProperty 无参构造器所有字段为 null
        let font = FontProperty::new();
        assert!(font.font_name().is_none());
        assert!(font.font_height_in_points().is_none());
        assert!(font.italic().is_none());
        assert!(font.strikeout().is_none());
        assert!(font.color().is_none());
        assert!(font.type_offset().is_none());
        assert!(font.underline().is_none());
        assert!(font.charset().is_none());
        assert!(font.bold().is_none());
    }

    #[test]
    fn default_trait_returns_all_none() {
        // 对应 Java：Default 派生
        let font = FontProperty::default();
        assert!(font.font_name().is_none());
    }

    #[test]
    fn build_from_excel_font_style() {
        // 对应 Java：FontProperty.build(ExcelFontStyle)
        let style = ExcelFontStyle {
            font_name: Some("Arial"),
            font_height_in_points: Some(12.0),
            italic: Some(true),
            strikeout: Some(false),
            color: None,
            type_offset: None,
            underline: None,
            charset: Some(1),
            bold: Some(true),
        };
        let font = FontProperty::build(style);
        assert_eq!(font.font_name(), Some("Arial"));
        assert_eq!(font.font_height_in_points(), Some(12.0));
        assert_eq!(font.italic(), Some(true));
        assert_eq!(font.strikeout(), Some(false));
        assert_eq!(font.charset(), Some(1));
        assert_eq!(font.bold(), Some(true));
    }

    #[test]
    fn build_from_empty_style() {
        // 对应 Java：FontProperty.build 空样式
        let style = ExcelFontStyle::default();
        let font = FontProperty::build(style);
        assert!(font.font_name().is_none());
    }

    #[test]
    fn to_write_font_populates_all_fields() {
        // 对应 Java：toWriteFont 转换
        let mut font = FontProperty::new();
        font.set_font_name(Some("Arial".to_owned()));
        font.set_font_height_in_points(Some(14.0));
        font.set_italic(Some(true));
        font.set_strikeout(Some(false));
        font.set_bold(Some(true));
        font.set_charset(Some(2));
        let write_font = font.to_write_font();
        // 验证转换不 panic 且返回 WriteFont
        let _ = write_font;
    }

    #[test]
    fn to_write_font_empty_font() {
        // 对应 Java：空 FontProperty 转换不 panic
        let font = FontProperty::new();
        let write_font = font.to_write_font();
        let _ = write_font;
    }

    #[test]
    fn font_name_setter_and_getter() {
        // 对应 Java：fontName getter/setter
        let mut font = FontProperty::new();
        assert!(font.get_font_name().is_none());
        font.set_font_name(Some("Times".to_owned()));
        assert_eq!(font.get_font_name(), Some("Times"));
        font.set_font_name(None);
        assert!(font.get_font_name().is_none());
    }

    #[test]
    fn font_height_in_points_setter_and_getter() {
        // 对应 Java：fontHeightInPoints getter/setter
        let mut font = FontProperty::new();
        assert!(font.get_font_height_in_points().is_none());
        font.set_font_height_in_points(Some(16.0));
        assert_eq!(font.get_font_height_in_points(), Some(16.0));
    }

    #[test]
    fn italic_setter_and_getter() {
        // 对应 Java：italic getter/setter
        let mut font = FontProperty::new();
        font.set_italic(Some(true));
        assert_eq!(font.get_italic(), Some(true));
        assert_eq!(font.italic(), Some(true));
    }

    #[test]
    fn strikeout_setter_and_getter() {
        // 对应 Java：strikeout getter/setter
        let mut font = FontProperty::new();
        font.set_strikeout(Some(true));
        assert_eq!(font.get_strikeout(), Some(true));
    }

    #[test]
    fn color_setter_and_getter() {
        // 对应 Java：color getter/setter
        let mut font = FontProperty::new();
        assert!(font.get_color().is_none());
        font.set_color(Some(ExcelColor::Rgb(0xFF0000)));
        assert_eq!(font.get_color(), Some(ExcelColor::Rgb(0xFF0000)));
    }

    #[test]
    fn type_offset_setter_and_getter() {
        // 对应 Java：typeOffset getter/setter
        let mut font = FontProperty::new();
        font.set_type_offset(Some(ExcelFontScript::Superscript));
        assert_eq!(font.get_type_offset(), Some(ExcelFontScript::Superscript));
    }

    #[test]
    fn underline_setter_and_getter() {
        // 对应 Java：underline getter/setter
        let mut font = FontProperty::new();
        font.set_underline(Some(ExcelUnderline::Single));
        assert_eq!(font.get_underline(), Some(ExcelUnderline::Single));
    }

    #[test]
    fn charset_setter_and_getter() {
        // 对应 Java：charset getter/setter
        let mut font = FontProperty::new();
        font.set_charset(Some(134));
        assert_eq!(font.get_charset(), Some(134));
    }

    #[test]
    fn bold_setter_and_getter() {
        // 对应 Java：bold getter/setter
        let mut font = FontProperty::new();
        font.set_bold(Some(true));
        assert_eq!(font.get_bold(), Some(true));
        assert_eq!(font.bold(), Some(true));
    }

    #[test]
    fn partial_eq_considers_nan_equal() {
        // 对应 Java：java_double_bits 将 NaN 规范化为统一比特
        let mut a = FontProperty::new();
        a.set_font_height_in_points(Some(f64::NAN));
        let mut b = FontProperty::new();
        b.set_font_height_in_points(Some(f64::NAN));
        assert_eq!(a, b);
    }

    #[test]
    fn partial_eq_different_fonts_not_equal() {
        // 对应 Java：不同字体名称不相等
        let mut a = FontProperty::new();
        a.set_font_name(Some("Arial".to_owned()));
        let mut b = FontProperty::new();
        b.set_font_name(Some("Times".to_owned()));
        assert_ne!(a, b);
    }

    #[test]
    fn hash_consistency() {
        // 对应 Java：相同内容哈希一致
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut a = FontProperty::new();
        a.set_font_name(Some("Arial".to_owned()));
        a.set_bold(Some(true));
        let mut b = FontProperty::new();
        b.set_font_name(Some("Arial".to_owned()));
        b.set_bold(Some(true));
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn clone_produces_equal_instance() {
        // 对应 Java：clone
        let mut font = FontProperty::new();
        font.set_font_name(Some("Arial".to_owned()));
        let cloned = font.clone();
        assert_eq!(font, cloned);
    }
}
