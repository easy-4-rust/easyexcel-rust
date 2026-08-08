//! 对应 Java：`com.alibaba.excel.metadata.property.FontProperty`.

use crate::core::excel_color::ExcelColor;
use crate::core::excel_font_script::ExcelFontScript;
use crate::core::excel_underline::ExcelUnderline;

/// 对应 Java：`FontProperty`. Rust reuses `ExcelFontStyle` for the
/// runtime representation; this struct exists for 1:1 Java package
/// parity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontProperty {
    /// Font family name. (Java `fontName`)
    pub font_name: Option<&'static str>,
    /// Font size in points. (Java `fontHeightInPoints`)
    pub font_height_in_points: Option<f64>,
    /// Italic. (Java `italic`)
    pub italic: Option<bool>,
    /// Strike-through. (Java `strikeout`)
    pub strikeout: Option<bool>,
    /// Color. (Java `color`)
    pub color: Option<ExcelColor>,
    /// Super/subscript. (Java `typeOffset`)
    pub type_offset: Option<ExcelFontScript>,
    /// Underline. (Java `underline`)
    pub underline: Option<ExcelUnderline>,
    /// Character set. (Java `charset`)
    pub charset: Option<u8>,
    /// Bold. (Java `bold`)
    pub bold: Option<bool>,
}

impl FontProperty {
    /// 创建空字体属性，所有字段保持 Java `null` 语义。
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

    /// 从注解等价字体样式构建属性。
    #[must_use]
    pub const fn build(style: crate::ExcelFontStyle) -> Self {
        Self {
            font_name: style.font_name,
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

    /// 返回字体名称。
    #[must_use]
    pub const fn font_name(&self) -> Option<&'static str> { self.font_name }
    /// 设置字体名称。
    pub const fn set_font_name(&mut self, value: Option<&'static str>) { self.font_name = value; }
    /// 返回字号（point）。
    #[must_use]
    pub const fn font_height_in_points(&self) -> Option<f64> { self.font_height_in_points }
    /// 设置字号（point）。
    pub const fn set_font_height_in_points(&mut self, value: Option<f64>) { self.font_height_in_points = value; }
    /// 返回斜体标志。
    #[must_use]
    pub const fn italic(&self) -> Option<bool> { self.italic }
    /// 设置斜体标志。
    pub const fn set_italic(&mut self, value: Option<bool>) { self.italic = value; }
    /// 返回删除线标志。
    #[must_use]
    pub const fn strikeout(&self) -> Option<bool> { self.strikeout }
    /// 设置删除线标志。
    pub const fn set_strikeout(&mut self, value: Option<bool>) { self.strikeout = value; }
    /// 返回字体颜色。
    #[must_use]
    pub const fn color(&self) -> Option<ExcelColor> { self.color }
    /// 设置字体颜色。
    pub const fn set_color(&mut self, value: Option<ExcelColor>) { self.color = value; }
    /// 返回上下标类型。
    #[must_use]
    pub const fn type_offset(&self) -> Option<ExcelFontScript> { self.type_offset }
    /// 设置上下标类型。
    pub const fn set_type_offset(&mut self, value: Option<ExcelFontScript>) { self.type_offset = value; }
    /// 返回下划线类型。
    #[must_use]
    pub const fn underline(&self) -> Option<ExcelUnderline> { self.underline }
    /// 设置下划线类型。
    pub const fn set_underline(&mut self, value: Option<ExcelUnderline>) { self.underline = value; }
    /// 返回字符集。
    #[must_use]
    pub const fn charset(&self) -> Option<u8> { self.charset }
    /// 设置字符集。
    pub const fn set_charset(&mut self, value: Option<u8>) { self.charset = value; }
    /// 返回粗体标志。
    #[must_use]
    pub const fn bold(&self) -> Option<bool> { self.bold }
    /// 设置粗体标志。
    pub const fn set_bold(&mut self, value: Option<bool>) { self.bold = value; }

    /// 转换为写入引擎使用的字体样式。
    #[must_use]
    pub const fn write_font(self) -> crate::ExcelFontStyle {
        crate::ExcelFontStyle {
            font_name: self.font_name,
            font_height_in_points: self.font_height_in_points,
            italic: self.italic,
            strikeout: self.strikeout,
            color: self.color,
            type_offset: self.type_offset,
            underline: self.underline,
            charset: self.charset,
            bold: self.bold,
        }
    }
}

impl Default for FontProperty {
    fn default() -> Self { Self::new() }
}
