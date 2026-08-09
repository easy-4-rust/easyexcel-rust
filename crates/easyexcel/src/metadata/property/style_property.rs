//! 对应 Java：`com.alibaba.excel.metadata.property.StyleProperty`。

use crate::core::{
    ExcelBorderStyle, ExcelCellStyle, ExcelColor, ExcelDataFormat, ExcelFillPattern,
    ExcelHorizontalAlignment, ExcelVerticalAlignment, WriteCellStyle, WriteFont,
};

/// 注解解析后的运行期单元格样式属性。
///
/// 对应 Java：`com.alibaba.excel.metadata.property.StyleProperty`。静态注解先由
/// [`ExcelCellStyle`] 承载；进入该对象后提升为拥有 `WriteFont` 的
/// [`WriteCellStyle`]，从而保留 Java setter 可写入动态字符串的语义。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct StyleProperty {
    write_cell_style: WriteCellStyle,
}

impl StyleProperty {
    /// 创建所有属性均未指定的对象。对应 Java 无参构造器。
    #[must_use]
    pub const fn new() -> Self {
        Self { write_cell_style: WriteCellStyle::new() }
    }

    /// Java 默认对象别名。
    #[must_use]
    pub const fn empty() -> Self { Self::new() }

    /// 从注解期轻量样式创建运行期属性。
    #[must_use]
    pub fn from_cell_style(cell_style: ExcelCellStyle) -> Self {
        Self { write_cell_style: cell_style.into() }
    }

    /// 从完整运行期样式创建属性。
    #[must_use]
    pub const fn from_write_cell_style(write_cell_style: WriteCellStyle) -> Self {
        Self { write_cell_style }
    }

    /// 返回完整运行期样式。
    #[must_use]
    pub const fn write_cell_style(&self) -> &WriteCellStyle { &self.write_cell_style }

    /// 消费属性并返回完整运行期样式。
    #[must_use]
    pub fn into_write_cell_style(self) -> WriteCellStyle { self.write_cell_style }

    /// Java `getDataFormatData`。
    #[must_use]
    pub const fn get_data_format_data(&self) -> Option<ExcelDataFormat> { self.write_cell_style.data_format }
    /// Java `setDataFormatData`。
    pub const fn set_data_format_data(&mut self, value: Option<ExcelDataFormat>) { self.write_cell_style.data_format = value; }
    /// Java `getWriteFont`。
    #[must_use]
    pub const fn get_write_font(&self) -> Option<&WriteFont> { self.write_cell_style.font.as_ref() }
    /// Java `setWriteFont`。
    pub fn set_write_font(&mut self, value: Option<WriteFont>) { self.write_cell_style.font = value; }
    /// Java `getHidden`。
    #[must_use]
    pub const fn get_hidden(&self) -> Option<bool> { self.write_cell_style.hidden }
    /// Java `setHidden`。
    pub const fn set_hidden(&mut self, value: Option<bool>) { self.write_cell_style.hidden = value; }
    /// Java `getLocked`。
    #[must_use]
    pub const fn get_locked(&self) -> Option<bool> { self.write_cell_style.locked }
    /// Java `setLocked`。
    pub const fn set_locked(&mut self, value: Option<bool>) { self.write_cell_style.locked = value; }
    /// Java `getQuotePrefix`。
    #[must_use]
    pub const fn get_quote_prefix(&self) -> Option<bool> { self.write_cell_style.quote_prefix }
    /// Java `setQuotePrefix`。
    pub const fn set_quote_prefix(&mut self, value: Option<bool>) { self.write_cell_style.quote_prefix = value; }
    /// Java `getHorizontalAlignment`。
    #[must_use]
    pub const fn get_horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> { self.write_cell_style.horizontal_alignment }
    /// Java `setHorizontalAlignment`。
    pub const fn set_horizontal_alignment(&mut self, value: Option<ExcelHorizontalAlignment>) { self.write_cell_style.horizontal_alignment = value; }
    /// Java `getWrapped`。
    #[must_use]
    pub const fn get_wrapped(&self) -> Option<bool> { self.write_cell_style.wrapped }
    /// Java `setWrapped`。
    pub const fn set_wrapped(&mut self, value: Option<bool>) { self.write_cell_style.wrapped = value; }
    /// Java `getVerticalAlignment`。
    #[must_use]
    pub const fn get_vertical_alignment(&self) -> Option<ExcelVerticalAlignment> { self.write_cell_style.vertical_alignment }
    /// Java `setVerticalAlignment`。
    pub const fn set_vertical_alignment(&mut self, value: Option<ExcelVerticalAlignment>) { self.write_cell_style.vertical_alignment = value; }
    /// Java `getRotation`。
    #[must_use]
    pub const fn get_rotation(&self) -> Option<i16> { self.write_cell_style.rotation }
    /// Java `setRotation`。
    pub const fn set_rotation(&mut self, value: Option<i16>) { self.write_cell_style.rotation = value; }
    /// Java `getIndent`。
    #[must_use]
    pub const fn get_indent(&self) -> Option<u8> { self.write_cell_style.indent }
    /// Java `setIndent`。
    pub const fn set_indent(&mut self, value: Option<u8>) { self.write_cell_style.indent = value; }
    /// Java `getBorderLeft`。
    #[must_use]
    pub const fn get_border_left(&self) -> Option<ExcelBorderStyle> { self.write_cell_style.border_left }
    /// Java `setBorderLeft`。
    pub const fn set_border_left(&mut self, value: Option<ExcelBorderStyle>) { self.write_cell_style.border_left = value; }
    /// Java `getBorderRight`。
    #[must_use]
    pub const fn get_border_right(&self) -> Option<ExcelBorderStyle> { self.write_cell_style.border_right }
    /// Java `setBorderRight`。
    pub const fn set_border_right(&mut self, value: Option<ExcelBorderStyle>) { self.write_cell_style.border_right = value; }
    /// Java `getBorderTop`。
    #[must_use]
    pub const fn get_border_top(&self) -> Option<ExcelBorderStyle> { self.write_cell_style.border_top }
    /// Java `setBorderTop`。
    pub const fn set_border_top(&mut self, value: Option<ExcelBorderStyle>) { self.write_cell_style.border_top = value; }
    /// Java `getBorderBottom`。
    #[must_use]
    pub const fn get_border_bottom(&self) -> Option<ExcelBorderStyle> { self.write_cell_style.border_bottom }
    /// Java `setBorderBottom`。
    pub const fn set_border_bottom(&mut self, value: Option<ExcelBorderStyle>) { self.write_cell_style.border_bottom = value; }
    /// Java `getLeftBorderColor`。
    #[must_use]
    pub const fn get_left_border_color(&self) -> Option<ExcelColor> { self.write_cell_style.left_border_color }
    /// Java `setLeftBorderColor`。
    pub const fn set_left_border_color(&mut self, value: Option<ExcelColor>) { self.write_cell_style.left_border_color = value; }
    /// Java `getRightBorderColor`。
    #[must_use]
    pub const fn get_right_border_color(&self) -> Option<ExcelColor> { self.write_cell_style.right_border_color }
    /// Java `setRightBorderColor`。
    pub const fn set_right_border_color(&mut self, value: Option<ExcelColor>) { self.write_cell_style.right_border_color = value; }
    /// Java `getTopBorderColor`。
    #[must_use]
    pub const fn get_top_border_color(&self) -> Option<ExcelColor> { self.write_cell_style.top_border_color }
    /// Java `setTopBorderColor`。
    pub const fn set_top_border_color(&mut self, value: Option<ExcelColor>) { self.write_cell_style.top_border_color = value; }
    /// Java `getBottomBorderColor`。
    #[must_use]
    pub const fn get_bottom_border_color(&self) -> Option<ExcelColor> { self.write_cell_style.bottom_border_color }
    /// Java `setBottomBorderColor`。
    pub const fn set_bottom_border_color(&mut self, value: Option<ExcelColor>) { self.write_cell_style.bottom_border_color = value; }
    /// Java `getFillPatternType`。
    #[must_use]
    pub const fn get_fill_pattern_type(&self) -> Option<ExcelFillPattern> { self.write_cell_style.fill_pattern }
    /// Java `setFillPatternType`。
    pub const fn set_fill_pattern_type(&mut self, value: Option<ExcelFillPattern>) { self.write_cell_style.fill_pattern = value; }
    /// Java `getFillBackgroundColor`。
    #[must_use]
    pub const fn get_fill_background_color(&self) -> Option<ExcelColor> { self.write_cell_style.fill_background_color }
    /// Java `setFillBackgroundColor`。
    pub const fn set_fill_background_color(&mut self, value: Option<ExcelColor>) { self.write_cell_style.fill_background_color = value; }
    /// Java `getFillForegroundColor`。
    #[must_use]
    pub const fn get_fill_foreground_color(&self) -> Option<ExcelColor> { self.write_cell_style.fill_foreground_color }
    /// Java `setFillForegroundColor`。
    pub const fn set_fill_foreground_color(&mut self, value: Option<ExcelColor>) { self.write_cell_style.fill_foreground_color = value; }
    /// Java `getShrinkToFit`。
    #[must_use]
    pub const fn get_shrink_to_fit(&self) -> Option<bool> { self.write_cell_style.shrink_to_fit }
    /// Java `setShrinkToFit`。
    pub const fn set_shrink_to_fit(&mut self, value: Option<bool>) { self.write_cell_style.shrink_to_fit = value; }

    /// Rust 风格隐藏标志 getter。
    #[must_use]
    pub const fn hidden(&self) -> Option<bool> { self.get_hidden() }
    /// Rust 风格锁定标志 getter。
    #[must_use]
    pub const fn locked(&self) -> Option<bool> { self.get_locked() }
    /// Rust 风格 quote-prefix getter。
    #[must_use]
    pub const fn quote_prefix(&self) -> Option<bool> { self.get_quote_prefix() }
    /// Rust 风格水平对齐 getter。
    #[must_use]
    pub const fn horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> { self.get_horizontal_alignment() }
    /// Rust 风格换行 getter。
    #[must_use]
    pub const fn wrapped(&self) -> Option<bool> { self.get_wrapped() }
    /// Rust 风格垂直对齐 getter。
    #[must_use]
    pub const fn vertical_alignment(&self) -> Option<ExcelVerticalAlignment> { self.get_vertical_alignment() }
    /// Rust 风格旋转角 getter。
    #[must_use]
    pub const fn rotation(&self) -> Option<i16> { self.get_rotation() }
    /// Rust 风格缩进 getter。
    #[must_use]
    pub const fn indent(&self) -> Option<u8> { self.get_indent() }
    /// Rust 风格左边框 getter。
    #[must_use]
    pub const fn border_left(&self) -> Option<ExcelBorderStyle> { self.get_border_left() }
    /// Rust 风格右边框 getter。
    #[must_use]
    pub const fn border_right(&self) -> Option<ExcelBorderStyle> { self.get_border_right() }
    /// Rust 风格上边框 getter。
    #[must_use]
    pub const fn border_top(&self) -> Option<ExcelBorderStyle> { self.get_border_top() }
    /// Rust 风格下边框 getter。
    #[must_use]
    pub const fn border_bottom(&self) -> Option<ExcelBorderStyle> { self.get_border_bottom() }
    /// Rust 风格左边框颜色 getter。
    #[must_use]
    pub const fn left_border_color(&self) -> Option<ExcelColor> { self.get_left_border_color() }
    /// Rust 风格右边框颜色 getter。
    #[must_use]
    pub const fn right_border_color(&self) -> Option<ExcelColor> { self.get_right_border_color() }
    /// Rust 风格上边框颜色 getter。
    #[must_use]
    pub const fn top_border_color(&self) -> Option<ExcelColor> { self.get_top_border_color() }
    /// Rust 风格下边框颜色 getter。
    #[must_use]
    pub const fn bottom_border_color(&self) -> Option<ExcelColor> { self.get_bottom_border_color() }
    /// Rust 风格填充图案 getter。
    #[must_use]
    pub const fn fill_pattern_type(&self) -> Option<ExcelFillPattern> { self.get_fill_pattern_type() }
    /// Rust 风格填充背景色 getter。
    #[must_use]
    pub const fn fill_background_color(&self) -> Option<ExcelColor> { self.get_fill_background_color() }
    /// Rust 风格填充前景色 getter。
    #[must_use]
    pub const fn fill_foreground_color(&self) -> Option<ExcelColor> { self.get_fill_foreground_color() }
    /// Rust 风格 shrink-to-fit getter。
    #[must_use]
    pub const fn shrink_to_fit(&self) -> Option<bool> { self.get_shrink_to_fit() }
    /// Rust 风格数字格式 getter。
    #[must_use]
    pub const fn data_format_data(&self) -> Option<ExcelDataFormat> { self.get_data_format_data() }
    /// Rust 风格字体 getter。
    #[must_use]
    pub const fn write_font(&self) -> Option<&WriteFont> { self.get_write_font() }
}
