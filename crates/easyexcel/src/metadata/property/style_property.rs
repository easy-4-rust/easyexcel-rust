//! 对应 Java：`com.alibaba.excel.metadata.property.StyleProperty`.

use crate::core::excel_cell_style::ExcelCellStyle;
use crate::core::{
    ExcelBorderStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern, ExcelFontStyle,
    ExcelHorizontalAlignment, ExcelVerticalAlignment,
};

/// 对应 Java：`StyleProperty`. Rust reuses `ExcelCellStyle` for the
/// runtime representation; this struct exists for 1:1 Java package
/// parity.
/// `Eq` is not derived because [`ExcelCellStyle`] embeds `f64` font size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StyleProperty {
    /// The underlying cell style. (Java delegates all fields)
    pub cell_style: ExcelCellStyle,
}

impl StyleProperty {
    /// 创建所有属性均未指定的 Java 默认对象。
    #[must_use]
    pub const fn empty() -> Self { Self { cell_style: ExcelCellStyle::new() } }
    /// Java `getDataFormatData` 别名。
    #[must_use] pub const fn get_data_format_data(&self) -> Option<ExcelDataFormat> { self.cell_style.data_format }
    /// Java `getWriteFont` 别名。
    #[must_use] pub const fn get_write_font(&self) -> Option<ExcelFontStyle> { self.cell_style.font }
    /// Java `getHidden` 别名。
    #[must_use] pub const fn get_hidden(&self) -> Option<bool> { self.cell_style.hidden }
    /// Java `getLocked` 别名。
    #[must_use] pub const fn get_locked(&self) -> Option<bool> { self.cell_style.locked }
    /// Java `getQuotePrefix` 别名。
    #[must_use] pub const fn get_quote_prefix(&self) -> Option<bool> { self.cell_style.quote_prefix }
    /// Java `getHorizontalAlignment` 别名。
    #[must_use] pub const fn get_horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> { self.cell_style.horizontal_alignment }
    /// Java `getWrapped` 别名。
    #[must_use] pub const fn get_wrapped(&self) -> Option<bool> { self.cell_style.wrapped }
    /// Java `getVerticalAlignment` 别名。
    #[must_use] pub const fn get_vertical_alignment(&self) -> Option<ExcelVerticalAlignment> { self.cell_style.vertical_alignment }
    /// Java `getRotation` 别名。
    #[must_use] pub const fn get_rotation(&self) -> Option<i16> { self.cell_style.rotation }
    /// Java `getIndent` 别名。
    #[must_use] pub const fn get_indent(&self) -> Option<u8> { self.cell_style.indent }
    /// Java `getBorderLeft` 别名。
    #[must_use] pub const fn get_border_left(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_left }
    /// Java `getBorderRight` 别名。
    #[must_use] pub const fn get_border_right(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_right }
    /// Java `getBorderTop` 别名。
    #[must_use] pub const fn get_border_top(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_top }
    /// Java `getBorderBottom` 别名。
    #[must_use] pub const fn get_border_bottom(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_bottom }
    /// Java `getLeftBorderColor` 别名。
    #[must_use] pub const fn get_left_border_color(&self) -> Option<ExcelColor> { self.cell_style.left_border_color }
    /// Java `getRightBorderColor` 别名。
    #[must_use] pub const fn get_right_border_color(&self) -> Option<ExcelColor> { self.cell_style.right_border_color }
    /// Java `getTopBorderColor` 别名。
    #[must_use] pub const fn get_top_border_color(&self) -> Option<ExcelColor> { self.cell_style.top_border_color }
    /// Java `getBottomBorderColor` 别名。
    #[must_use] pub const fn get_bottom_border_color(&self) -> Option<ExcelColor> { self.cell_style.bottom_border_color }
    /// Java `getFillPatternType` 别名。
    #[must_use] pub const fn get_fill_pattern_type(&self) -> Option<ExcelFillPattern> { self.cell_style.fill_pattern }
    /// Java `getFillBackgroundColor` 别名。
    #[must_use] pub const fn get_fill_background_color(&self) -> Option<ExcelColor> { self.cell_style.fill_background_color }
    /// Java `getFillForegroundColor` 别名。
    #[must_use] pub const fn get_fill_foreground_color(&self) -> Option<ExcelColor> { self.cell_style.fill_foreground_color }
    /// Java `getShrinkToFit` 别名。
    #[must_use] pub const fn get_shrink_to_fit(&self) -> Option<bool> { self.cell_style.shrink_to_fit }

    /// Creates a `StyleProperty`. (Java constructor)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.property.StyleProperty。
    pub const fn new(cell_style: ExcelCellStyle) -> Self {
        Self { cell_style }
    }

    /// 构建底层写样式，语义对应 Java `StyleProperty#build`。
    #[must_use]
    pub const fn build(self) -> ExcelCellStyle { self.cell_style }
    /// 返回隐藏标志。
    #[must_use]
    pub const fn hidden(&self) -> Option<bool> { self.cell_style.hidden }
    /// 设置隐藏标志。
    pub const fn set_hidden(&mut self, value: Option<bool>) { self.cell_style.hidden = value; }
    /// 返回锁定标志。
    #[must_use]
    pub const fn locked(&self) -> Option<bool> { self.cell_style.locked }
    /// 设置锁定标志。
    pub const fn set_locked(&mut self, value: Option<bool>) { self.cell_style.locked = value; }
    /// 返回 quote-prefix 标志。
    #[must_use]
    pub const fn quote_prefix(&self) -> Option<bool> { self.cell_style.quote_prefix }
    /// 设置 quote-prefix 标志。
    pub const fn set_quote_prefix(&mut self, value: Option<bool>) { self.cell_style.quote_prefix = value; }
    /// 返回水平对齐。
    #[must_use]
    pub const fn horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> { self.cell_style.horizontal_alignment }
    /// 设置水平对齐。
    pub const fn set_horizontal_alignment(&mut self, value: Option<ExcelHorizontalAlignment>) { self.cell_style.horizontal_alignment = value; }
    /// 返回换行标志。
    #[must_use]
    pub const fn wrapped(&self) -> Option<bool> { self.cell_style.wrapped }
    /// 设置换行标志。
    pub const fn set_wrapped(&mut self, value: Option<bool>) { self.cell_style.wrapped = value; }
    /// 返回垂直对齐。
    #[must_use]
    pub const fn vertical_alignment(&self) -> Option<ExcelVerticalAlignment> { self.cell_style.vertical_alignment }
    /// 设置垂直对齐。
    pub const fn set_vertical_alignment(&mut self, value: Option<ExcelVerticalAlignment>) { self.cell_style.vertical_alignment = value; }
    /// 返回旋转角。
    #[must_use]
    pub const fn rotation(&self) -> Option<i16> { self.cell_style.rotation }
    /// 设置旋转角。
    pub const fn set_rotation(&mut self, value: Option<i16>) { self.cell_style.rotation = value; }
    /// 返回缩进。
    #[must_use]
    pub const fn indent(&self) -> Option<u8> { self.cell_style.indent }
    /// 设置缩进。
    pub const fn set_indent(&mut self, value: Option<u8>) { self.cell_style.indent = value; }
    /// 返回左边框。
    #[must_use]
    pub const fn border_left(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_left }
    /// 设置左边框。
    pub const fn set_border_left(&mut self, value: Option<ExcelBorderStyle>) { self.cell_style.border_left = value; }
    /// 返回右边框。
    #[must_use]
    pub const fn border_right(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_right }
    /// 设置右边框。
    pub const fn set_border_right(&mut self, value: Option<ExcelBorderStyle>) { self.cell_style.border_right = value; }
    /// 返回上边框。
    #[must_use]
    pub const fn border_top(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_top }
    /// 设置上边框。
    pub const fn set_border_top(&mut self, value: Option<ExcelBorderStyle>) { self.cell_style.border_top = value; }
    /// 返回下边框。
    #[must_use]
    pub const fn border_bottom(&self) -> Option<ExcelBorderStyle> { self.cell_style.border_bottom }
    /// 设置下边框。
    pub const fn set_border_bottom(&mut self, value: Option<ExcelBorderStyle>) { self.cell_style.border_bottom = value; }
    /// 返回左边框颜色。
    #[must_use]
    pub const fn left_border_color(&self) -> Option<ExcelColor> { self.cell_style.left_border_color }
    /// 设置左边框颜色。
    pub const fn set_left_border_color(&mut self, value: Option<ExcelColor>) { self.cell_style.left_border_color = value; }
    /// 返回右边框颜色。
    #[must_use]
    pub const fn right_border_color(&self) -> Option<ExcelColor> { self.cell_style.right_border_color }
    /// 设置右边框颜色。
    pub const fn set_right_border_color(&mut self, value: Option<ExcelColor>) { self.cell_style.right_border_color = value; }
    /// 返回上边框颜色。
    #[must_use]
    pub const fn top_border_color(&self) -> Option<ExcelColor> { self.cell_style.top_border_color }
    /// 设置上边框颜色。
    pub const fn set_top_border_color(&mut self, value: Option<ExcelColor>) { self.cell_style.top_border_color = value; }
    /// 返回下边框颜色。
    #[must_use]
    pub const fn bottom_border_color(&self) -> Option<ExcelColor> { self.cell_style.bottom_border_color }
    /// 设置下边框颜色。
    pub const fn set_bottom_border_color(&mut self, value: Option<ExcelColor>) { self.cell_style.bottom_border_color = value; }
    /// 返回填充图案。
    #[must_use]
    pub const fn fill_pattern_type(&self) -> Option<ExcelFillPattern> { self.cell_style.fill_pattern }
    /// 设置填充图案。
    pub const fn set_fill_pattern_type(&mut self, value: Option<ExcelFillPattern>) { self.cell_style.fill_pattern = value; }
    /// 返回填充背景色。
    #[must_use]
    pub const fn fill_background_color(&self) -> Option<ExcelColor> { self.cell_style.fill_background_color }
    /// 设置填充背景色。
    pub const fn set_fill_background_color(&mut self, value: Option<ExcelColor>) { self.cell_style.fill_background_color = value; }
    /// 返回填充前景色。
    #[must_use]
    pub const fn fill_foreground_color(&self) -> Option<ExcelColor> { self.cell_style.fill_foreground_color }
    /// 设置填充前景色。
    pub const fn set_fill_foreground_color(&mut self, value: Option<ExcelColor>) { self.cell_style.fill_foreground_color = value; }
    /// 返回 shrink-to-fit 标志。
    #[must_use]
    pub const fn shrink_to_fit(&self) -> Option<bool> { self.cell_style.shrink_to_fit }
    /// 设置 shrink-to-fit 标志。
    pub const fn set_shrink_to_fit(&mut self, value: Option<bool>) { self.cell_style.shrink_to_fit = value; }
    /// 返回数字格式。
    #[must_use]
    pub const fn data_format_data(&self) -> Option<ExcelDataFormat> { self.cell_style.data_format }
    /// 设置数字格式。
    pub const fn set_data_format_data(&mut self, value: Option<ExcelDataFormat>) { self.cell_style.data_format = value; }
    /// 返回字体。
    #[must_use]
    pub const fn write_font(&self) -> Option<ExcelFontStyle> { self.cell_style.font }
    /// 设置字体。
    pub const fn set_write_font(&mut self, value: Option<ExcelFontStyle>) { self.cell_style.font = value; }
}
