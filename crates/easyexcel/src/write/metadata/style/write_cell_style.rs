//! 对应 Java：`com.alibaba.excel.write.metadata.style.WriteCellStyle`.

use crate::core::excel_border_style::ExcelBorderStyle;
use crate::core::excel_color::ExcelColor;
use crate::core::excel_data_format::ExcelDataFormat;
use crate::core::excel_fill_pattern::ExcelFillPattern;
use crate::core::excel_cell_style::ExcelCellStyle;
use crate::core::excel_font_style::ExcelFontStyle;
use crate::core::excel_horizontal_alignment::ExcelHorizontalAlignment;
use crate::core::excel_vertical_alignment::ExcelVerticalAlignment;
use crate::write::metadata::style::write_font::{
    WriteFont, merge_write_font, write_font_from_excel_font_style,
};

/// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。 Cell-style properties generated from `HeadStyle` or `ContentStyle` equivalents.
///
/// Fields correspond to Java's `WriteCellStyle`. Java's boxed `Short` /
/// `Integer` becomes `Option<u16>` / `Option<i16>`; Java's `BooleanEnum`
/// becomes `Option<bool>`. Nested `writeFont` is the owned [`WriteFont`]
/// runtime object; the copyable annotation model remains [`ExcelFontStyle`].
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct WriteCellStyle {
    /// Whether the cell is hidden when the sheet is protected.
    pub hidden: Option<bool>,
    /// Whether the cell is locked when the sheet is protected.
    pub locked: Option<bool>,
    /// Whether Excel treats the value as explicitly quoted text.
    pub quote_prefix: Option<bool>,
    /// Horizontal alignment.
    pub horizontal_alignment: Option<ExcelHorizontalAlignment>,
    /// Whether text wraps within the cell.
    pub wrapped: Option<bool>,
    /// Vertical alignment.
    pub vertical_alignment: Option<ExcelVerticalAlignment>,
    /// Text rotation in degrees.
    pub rotation: Option<i16>,
    /// Text indentation level.
    pub indent: Option<u8>,
    /// Left border style.
    pub border_left: Option<ExcelBorderStyle>,
    /// Right border style.
    pub border_right: Option<ExcelBorderStyle>,
    /// Top border style.
    pub border_top: Option<ExcelBorderStyle>,
    /// Bottom border style.
    pub border_bottom: Option<ExcelBorderStyle>,
    /// Left border indexed or RGB color.
    pub left_border_color: Option<ExcelColor>,
    /// Right border indexed or RGB color.
    pub right_border_color: Option<ExcelColor>,
    /// Top border indexed or RGB color.
    pub top_border_color: Option<ExcelColor>,
    /// Bottom border indexed or RGB color.
    pub bottom_border_color: Option<ExcelColor>,
    /// Fill pattern.
    pub fill_pattern: Option<ExcelFillPattern>,
    /// Fill background indexed or RGB color.
    pub fill_background_color: Option<ExcelColor>,
    /// Fill foreground indexed or RGB color.
    pub fill_foreground_color: Option<ExcelColor>,
    /// Whether text shrinks to fit the cell.
    pub shrink_to_fit: Option<bool>,
    /// Built-in or custom Excel number format.
    pub data_format: Option<ExcelDataFormat>,
    /// Nested font. (Java `WriteCellStyle.writeFont` / `WriteFont`)
    pub font: Option<WriteFont>,
}

impl WriteCellStyle {
    /// Java `getDataFormatData` 别名。
    #[must_use] pub const fn get_data_format_data(&self) -> Option<ExcelDataFormat> { self.data_format }
    /// Java `getWriteFont` 别名。
    #[must_use] pub const fn get_write_font(&self) -> Option<&WriteFont> { self.font.as_ref() }
    /// Java `getHidden` 别名。
    #[must_use] pub const fn get_hidden(&self) -> Option<bool> { self.hidden }
    /// Java `getLocked` 别名。
    #[must_use] pub const fn get_locked(&self) -> Option<bool> { self.locked }
    /// Java `getQuotePrefix` 别名。
    #[must_use] pub const fn get_quote_prefix(&self) -> Option<bool> { self.quote_prefix }
    /// Java `getHorizontalAlignment` 别名。
    #[must_use] pub const fn get_horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> { self.horizontal_alignment }
    /// Java `getWrapped` 别名。
    #[must_use] pub const fn get_wrapped(&self) -> Option<bool> { self.wrapped }
    /// Java `getVerticalAlignment` 别名。
    #[must_use] pub const fn get_vertical_alignment(&self) -> Option<ExcelVerticalAlignment> { self.vertical_alignment }
    /// Java `getRotation` 别名。
    #[must_use] pub const fn get_rotation(&self) -> Option<i16> { self.rotation }
    /// Java `getIndent` 别名。
    #[must_use] pub const fn get_indent(&self) -> Option<u8> { self.indent }
    /// Java `getBorderLeft` 别名。
    #[must_use] pub const fn get_border_left(&self) -> Option<ExcelBorderStyle> { self.border_left }
    /// Java `getBorderRight` 别名。
    #[must_use] pub const fn get_border_right(&self) -> Option<ExcelBorderStyle> { self.border_right }
    /// Java `getBorderTop` 别名。
    #[must_use] pub const fn get_border_top(&self) -> Option<ExcelBorderStyle> { self.border_top }
    /// Java `getBorderBottom` 别名。
    #[must_use] pub const fn get_border_bottom(&self) -> Option<ExcelBorderStyle> { self.border_bottom }
    /// Java `getLeftBorderColor` 别名。
    #[must_use] pub const fn get_left_border_color(&self) -> Option<ExcelColor> { self.left_border_color }
    /// Java `getRightBorderColor` 别名。
    #[must_use] pub const fn get_right_border_color(&self) -> Option<ExcelColor> { self.right_border_color }
    /// Java `getTopBorderColor` 别名。
    #[must_use] pub const fn get_top_border_color(&self) -> Option<ExcelColor> { self.top_border_color }
    /// Java `getBottomBorderColor` 别名。
    #[must_use] pub const fn get_bottom_border_color(&self) -> Option<ExcelColor> { self.bottom_border_color }
    /// Java `getFillPatternType` 别名。
    #[must_use] pub const fn get_fill_pattern_type(&self) -> Option<ExcelFillPattern> { self.fill_pattern }
    /// Java `getFillBackgroundColor` 别名。
    #[must_use] pub const fn get_fill_background_color(&self) -> Option<ExcelColor> { self.fill_background_color }
    /// Java `getFillForegroundColor` 别名。
    #[must_use] pub const fn get_fill_foreground_color(&self) -> Option<ExcelColor> { self.fill_foreground_color }
    /// Java `getShrinkToFit` 别名。
    #[must_use] pub const fn get_shrink_to_fit(&self) -> Option<bool> { self.shrink_to_fit }

    /// Creates an annotation style with every property unspecified. (Java `WriteCellStyle()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub const fn new() -> Self {
        Self {
            hidden: None,
            locked: None,
            quote_prefix: None,
            horizontal_alignment: None,
            wrapped: None,
            vertical_alignment: None,
            rotation: None,
            indent: None,
            border_left: None,
            border_right: None,
            border_top: None,
            border_bottom: None,
            left_border_color: None,
            right_border_color: None,
            top_border_color: None,
            bottom_border_color: None,
            fill_pattern: None,
            fill_background_color: None,
            fill_foreground_color: None,
            shrink_to_fit: None,
            data_format: None,
            font: None,
        }
    }

    /// Attaches a nested font. (Java `WriteCellStyle.setWriteFont(WriteFont)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.write.metadata.style.WriteCellStyle。
    pub fn with_font(mut self, font: WriteFont) -> Self {
        self.font = Some(font);
        self
    }

    /// 附加注解期字体并转为拥有所有权的运行期字体。
    #[must_use]
    pub fn with_excel_font_style(self, font: ExcelFontStyle) -> Self {
        self.with_font(write_font_from_excel_font_style(font))
    }

    /// 从注解解析出的样式与字体属性构建运行期单元格样式。
    ///
    /// 对应 Java：`WriteCellStyle.build(StyleProperty, FontProperty)`。两个参数
    /// 都缺失时返回 `None`，等价于 Java `null`；字体属性覆盖样式对象中原有字体。
    #[must_use]
    pub fn build(
        style_property: Option<&crate::StyleProperty>,
        font_property: Option<&crate::FontProperty>,
    ) -> Option<Self> {
        if style_property.is_none() && font_property.is_none() {
            return None;
        }
        let mut result = match style_property {
            Some(property) => property.write_cell_style().clone(),
            None => Self::new(),
        };
        if let Some(property) = font_property {
            result.font = Some(property.to_write_font());
        }
        Some(result)
    }
    /// 返回隐藏标志。
    #[must_use]
    pub const fn hidden(&self) -> Option<bool> { self.hidden }
    /// 设置隐藏标志。
    pub const fn set_hidden(&mut self, value: Option<bool>) { self.hidden = value; }
    /// 返回锁定标志。
    #[must_use]
    pub const fn locked(&self) -> Option<bool> { self.locked }
    /// 设置锁定标志。
    pub const fn set_locked(&mut self, value: Option<bool>) { self.locked = value; }
    /// 返回 quote-prefix。
    #[must_use]
    pub const fn quote_prefix(&self) -> Option<bool> { self.quote_prefix }
    /// 设置 quote-prefix。
    pub const fn set_quote_prefix(&mut self, value: Option<bool>) { self.quote_prefix = value; }
    /// 返回水平对齐。
    #[must_use]
    pub const fn horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> { self.horizontal_alignment }
    /// 设置水平对齐。
    pub const fn set_horizontal_alignment(&mut self, value: Option<ExcelHorizontalAlignment>) { self.horizontal_alignment = value; }
    /// 返回换行标志。
    #[must_use]
    pub const fn wrapped(&self) -> Option<bool> { self.wrapped }
    /// 设置换行标志。
    pub const fn set_wrapped(&mut self, value: Option<bool>) { self.wrapped = value; }
    /// 返回垂直对齐。
    #[must_use]
    pub const fn vertical_alignment(&self) -> Option<ExcelVerticalAlignment> { self.vertical_alignment }
    /// 设置垂直对齐。
    pub const fn set_vertical_alignment(&mut self, value: Option<ExcelVerticalAlignment>) { self.vertical_alignment = value; }
    /// 返回旋转角。
    #[must_use]
    pub const fn rotation(&self) -> Option<i16> { self.rotation }
    /// 设置旋转角。
    pub const fn set_rotation(&mut self, value: Option<i16>) { self.rotation = value; }
    /// 返回缩进。
    #[must_use]
    pub const fn indent(&self) -> Option<u8> { self.indent }
    /// 设置缩进。
    pub const fn set_indent(&mut self, value: Option<u8>) { self.indent = value; }
    /// 返回左边框。
    #[must_use]
    pub const fn border_left(&self) -> Option<ExcelBorderStyle> { self.border_left }
    /// 设置左边框。
    pub const fn set_border_left(&mut self, value: Option<ExcelBorderStyle>) { self.border_left = value; }
    /// 返回右边框。
    #[must_use]
    pub const fn border_right(&self) -> Option<ExcelBorderStyle> { self.border_right }
    /// 设置右边框。
    pub const fn set_border_right(&mut self, value: Option<ExcelBorderStyle>) { self.border_right = value; }
    /// 返回上边框。
    #[must_use]
    pub const fn border_top(&self) -> Option<ExcelBorderStyle> { self.border_top }
    /// 设置上边框。
    pub const fn set_border_top(&mut self, value: Option<ExcelBorderStyle>) { self.border_top = value; }
    /// 返回下边框。
    #[must_use]
    pub const fn border_bottom(&self) -> Option<ExcelBorderStyle> { self.border_bottom }
    /// 设置下边框。
    pub const fn set_border_bottom(&mut self, value: Option<ExcelBorderStyle>) { self.border_bottom = value; }
    /// 返回左边框颜色。
    #[must_use]
    pub const fn left_border_color(&self) -> Option<ExcelColor> { self.left_border_color }
    /// 设置左边框颜色。
    pub const fn set_left_border_color(&mut self, value: Option<ExcelColor>) { self.left_border_color = value; }
    /// 返回右边框颜色。
    #[must_use]
    pub const fn right_border_color(&self) -> Option<ExcelColor> { self.right_border_color }
    /// 设置右边框颜色。
    pub const fn set_right_border_color(&mut self, value: Option<ExcelColor>) { self.right_border_color = value; }
    /// 返回上边框颜色。
    #[must_use]
    pub const fn top_border_color(&self) -> Option<ExcelColor> { self.top_border_color }
    /// 设置上边框颜色。
    pub const fn set_top_border_color(&mut self, value: Option<ExcelColor>) { self.top_border_color = value; }
    /// 返回下边框颜色。
    #[must_use]
    pub const fn bottom_border_color(&self) -> Option<ExcelColor> { self.bottom_border_color }
    /// 设置下边框颜色。
    pub const fn set_bottom_border_color(&mut self, value: Option<ExcelColor>) { self.bottom_border_color = value; }
    /// 返回填充图案。
    #[must_use]
    pub const fn fill_pattern_type(&self) -> Option<ExcelFillPattern> { self.fill_pattern }
    /// 设置填充图案。
    pub const fn set_fill_pattern_type(&mut self, value: Option<ExcelFillPattern>) { self.fill_pattern = value; }
    /// 返回填充背景色。
    #[must_use]
    pub const fn fill_background_color(&self) -> Option<ExcelColor> { self.fill_background_color }
    /// 设置填充背景色。
    pub const fn set_fill_background_color(&mut self, value: Option<ExcelColor>) { self.fill_background_color = value; }
    /// 返回填充前景色。
    #[must_use]
    pub const fn fill_foreground_color(&self) -> Option<ExcelColor> { self.fill_foreground_color }
    /// 设置填充前景色。
    pub const fn set_fill_foreground_color(&mut self, value: Option<ExcelColor>) { self.fill_foreground_color = value; }
    /// 返回 shrink-to-fit。
    #[must_use]
    pub const fn shrink_to_fit(&self) -> Option<bool> { self.shrink_to_fit }
    /// 设置 shrink-to-fit。
    pub const fn set_shrink_to_fit(&mut self, value: Option<bool>) { self.shrink_to_fit = value; }
    /// 返回数字格式。
    #[must_use]
    pub const fn data_format_data(&self) -> Option<ExcelDataFormat> { self.data_format }
    /// 设置数字格式。
    pub const fn set_data_format_data(&mut self, value: Option<ExcelDataFormat>) { self.data_format = value; }
    /// 返回字体。
    #[must_use]
    pub const fn write_font(&self) -> Option<&WriteFont> { self.font.as_ref() }
    /// 设置字体。
    pub fn set_write_font(&mut self, value: Option<WriteFont>) { self.font = value; }
    /// 合并源样式的非空字段到目标样式。
    ///
    /// 对应 Java 静态 `merge(source, target)` 的原位副作用。
    pub fn merge(source: &Self, target: &mut Self) {
        *target = merge_write_cell_style(source, target.clone());
    }

    /// 返回合并后的值，供 Rust 值式调用链使用。
    #[must_use]
    pub fn merged(source: &Self, target: Self) -> Self {
        merge_write_cell_style(source, target)
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn with_font_attaches_nested_font() {
        // 对应 Java：WriteCellStyle.setWriteFont
        let font = ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::default()
        };
        let expected = write_font_from_excel_font_style(font.clone());
        let style = WriteCellStyle::new().with_excel_font_style(font);
        assert_eq!(style.font, Some(expected));
        let plain = WriteCellStyle::new();
        assert_eq!(plain.font, None);
    }
}

/// 对应 Java：`WriteCellStyle.merge(WriteCellStyle source, WriteCellStyle target)`.
///
/// Java merges the source's non-null fields into the target, including
/// nested `WriteFont.merge`. The Rust port performs the same union over
/// [`WriteCellStyle`]'s `Option` fields and [`WriteCellStyle::font`].
#[must_use]
pub fn merge_write_cell_style(
    source: &WriteCellStyle,
    mut target: WriteCellStyle,
) -> WriteCellStyle {
    macro_rules! or {
        ($field:ident) => {
            if source.$field.is_some() {
                target.$field = source.$field;
            }
        };
    }
    or!(hidden);
    or!(locked);
    or!(quote_prefix);
    or!(horizontal_alignment);
    or!(wrapped);
    or!(vertical_alignment);
    or!(rotation);
    or!(indent);
    or!(border_left);
    or!(border_right);
    or!(border_top);
    or!(border_bottom);
    or!(left_border_color);
    or!(right_border_color);
    or!(top_border_color);
    or!(bottom_border_color);
    or!(fill_pattern);
    or!(fill_background_color);
    or!(fill_foreground_color);
    or!(shrink_to_fit);
    or!(data_format);
    // Java `WriteFont.merge(source.getWriteFont(), target.getWriteFont())`
    if let Some(source_font) = &source.font {
        target.font = Some(match target.font {
            Some(existing) => merge_write_font(source_font, existing),
            None => source_font.clone(),
        });
    }
    target
}

impl From<ExcelCellStyle> for WriteCellStyle {
    /// 把注解期轻量样式提升为 Java 运行期样式，静态字体名称转为拥有所有权的字符串。
    fn from(style: ExcelCellStyle) -> Self {
        Self {
            hidden: style.hidden,
            locked: style.locked,
            quote_prefix: style.quote_prefix,
            horizontal_alignment: style.horizontal_alignment,
            wrapped: style.wrapped,
            vertical_alignment: style.vertical_alignment,
            rotation: style.rotation,
            indent: style.indent,
            border_left: style.border_left,
            border_right: style.border_right,
            border_top: style.border_top,
            border_bottom: style.border_bottom,
            left_border_color: style.left_border_color,
            right_border_color: style.right_border_color,
            top_border_color: style.top_border_color,
            bottom_border_color: style.bottom_border_color,
            fill_pattern: style.fill_pattern,
            fill_background_color: style.fill_background_color,
            fill_foreground_color: style.fill_foreground_color,
            shrink_to_fit: style.shrink_to_fit,
            data_format: style.data_format,
            font: style.font.map(write_font_from_excel_font_style),
        }
    }
}

impl WriteCellStyle {
    /// 返回不含运行期字体的引擎轻量字段。
    ///
    /// 字体由 XLS/XLSX 边界直接读取 [`Self::write_font`] 并应用，避免把动态
    /// `String` 反向收窄为静态字符串。
    #[must_use]
    pub const fn engine_cell_style(&self) -> ExcelCellStyle {
        ExcelCellStyle {
            hidden: self.hidden,
            locked: self.locked,
            quote_prefix: self.quote_prefix,
            horizontal_alignment: self.horizontal_alignment,
            wrapped: self.wrapped,
            vertical_alignment: self.vertical_alignment,
            rotation: self.rotation,
            indent: self.indent,
            border_left: self.border_left,
            border_right: self.border_right,
            border_top: self.border_top,
            border_bottom: self.border_bottom,
            left_border_color: self.left_border_color,
            right_border_color: self.right_border_color,
            top_border_color: self.top_border_color,
            bottom_border_color: self.bottom_border_color,
            fill_pattern: self.fill_pattern,
            fill_background_color: self.fill_background_color,
            fill_foreground_color: self.fill_foreground_color,
            shrink_to_fit: self.shrink_to_fit,
            data_format: self.data_format,
            font: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{
        ExcelBorderStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern, ExcelFontStyle,
        ExcelHorizontalAlignment, ExcelVerticalAlignment,
    };

    use super::*;

    #[test]
    fn merge_empty_source_preserves_target() {
        let source = WriteCellStyle::new();
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged, WriteCellStyle::new());
    }

    #[test]
    fn merge_copies_hidden_field() {
        let source = WriteCellStyle {
            hidden: Some(true),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.hidden, Some(true));
    }

    #[test]
    fn merge_copies_locked_field() {
        let source = WriteCellStyle {
            locked: Some(true),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.locked, Some(true));
    }

    #[test]
    fn merge_copies_quote_prefix_field() {
        let source = WriteCellStyle {
            quote_prefix: Some(true),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.quote_prefix, Some(true));
    }

    #[test]
    fn merge_copies_horizontal_alignment() {
        let source = WriteCellStyle {
            horizontal_alignment: Some(ExcelHorizontalAlignment::Center),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(
            merged.horizontal_alignment,
            Some(ExcelHorizontalAlignment::Center)
        );
    }

    #[test]
    fn merge_copies_wrapped_field() {
        let source = WriteCellStyle {
            wrapped: Some(true),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.wrapped, Some(true));
    }

    #[test]
    fn merge_copies_vertical_alignment() {
        let source = WriteCellStyle {
            vertical_alignment: Some(ExcelVerticalAlignment::Center),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(
            merged.vertical_alignment,
            Some(ExcelVerticalAlignment::Center)
        );
    }

    #[test]
    fn merge_copies_rotation_field() {
        let source = WriteCellStyle {
            rotation: Some(45),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.rotation, Some(45));
    }

    #[test]
    fn merge_copies_indent_field() {
        let source = WriteCellStyle {
            indent: Some(2),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.indent, Some(2));
    }

    #[test]
    fn merge_copies_border_fields() {
        let source = WriteCellStyle {
            border_left: Some(ExcelBorderStyle::Thin),
            border_right: Some(ExcelBorderStyle::Medium),
            border_top: Some(ExcelBorderStyle::Dashed),
            border_bottom: Some(ExcelBorderStyle::Dotted),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.border_left, Some(ExcelBorderStyle::Thin));
        assert_eq!(merged.border_right, Some(ExcelBorderStyle::Medium));
        assert_eq!(merged.border_top, Some(ExcelBorderStyle::Dashed));
        assert_eq!(merged.border_bottom, Some(ExcelBorderStyle::Dotted));
    }

    #[test]
    fn merge_copies_border_colors() {
        let source = WriteCellStyle {
            left_border_color: Some(ExcelColor::Indexed(1)),
            right_border_color: Some(ExcelColor::Indexed(2)),
            top_border_color: Some(ExcelColor::Indexed(3)),
            bottom_border_color: Some(ExcelColor::Indexed(4)),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.left_border_color, Some(ExcelColor::Indexed(1)));
        assert_eq!(merged.right_border_color, Some(ExcelColor::Indexed(2)));
        assert_eq!(merged.top_border_color, Some(ExcelColor::Indexed(3)));
        assert_eq!(merged.bottom_border_color, Some(ExcelColor::Indexed(4)));
    }

    #[test]
    fn merge_copies_fill_pattern_and_colors() {
        let source = WriteCellStyle {
            fill_pattern: Some(ExcelFillPattern::Solid),
            fill_background_color: Some(ExcelColor::Indexed(10)),
            fill_foreground_color: Some(ExcelColor::Indexed(20)),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.fill_pattern, Some(ExcelFillPattern::Solid));
        assert_eq!(merged.fill_background_color, Some(ExcelColor::Indexed(10)));
        assert_eq!(merged.fill_foreground_color, Some(ExcelColor::Indexed(20)));
    }

    #[test]
    fn merge_copies_shrink_to_fit() {
        let source = WriteCellStyle {
            shrink_to_fit: Some(true),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.shrink_to_fit, Some(true));
    }

    #[test]
    fn merge_copies_data_format() {
        let source = WriteCellStyle {
            data_format: Some(ExcelDataFormat::Builtin(0)),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.data_format, Some(ExcelDataFormat::Builtin(0)));
    }

    #[test]
    fn merge_copies_font_when_target_has_none() {
        let source_font = ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::default()
        };
        let source = WriteCellStyle {
            font: Some(write_font_from_excel_font_style(source_font)),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle::new();
        let merged = merge_write_cell_style(&source, target);
        assert!(merged.font.is_some());
        assert_eq!(merged.font.unwrap().get_bold(), Some(true));
    }

    #[test]
    fn merge_merges_font_when_target_has_font() {
        let source_font = ExcelFontStyle {
            bold: Some(true),
            ..ExcelFontStyle::default()
        };
        let target_font = ExcelFontStyle {
            italic: Some(true),
            ..ExcelFontStyle::default()
        };
        let source = WriteCellStyle {
            font: Some(write_font_from_excel_font_style(source_font)),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle {
            font: Some(write_font_from_excel_font_style(target_font)),
            ..WriteCellStyle::new()
        };
        let merged = merge_write_cell_style(&source, target);
        let font = merged.font.unwrap();
        assert_eq!(font.get_bold(), Some(true));
        assert_eq!(font.get_italic(), Some(true));
    }

    #[test]
    fn merge_overwrites_target_when_source_has_value() {
        let source = WriteCellStyle {
            hidden: Some(true),
            ..WriteCellStyle::new()
        };
        let target = WriteCellStyle {
            hidden: Some(false),
            ..WriteCellStyle::new()
        };
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.hidden, Some(true));
    }

    #[test]
    fn merge_preserves_target_when_source_field_is_none() {
        let source = WriteCellStyle::new();
        let target = WriteCellStyle {
            hidden: Some(true),
            ..WriteCellStyle::new()
        };
        let merged = merge_write_cell_style(&source, target);
        assert_eq!(merged.hidden, Some(true));
    }

    #[test]
    fn write_cell_style_is_excel_cell_style_alias() {
        let style = WriteCellStyle::new();
        let alias: WriteCellStyle = style;
        assert_eq!(alias, WriteCellStyle::new());
    }
}
