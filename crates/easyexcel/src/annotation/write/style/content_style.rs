//! 对应 Java：`com.alibaba.excel.annotation.write.style.ContentStyle`。

use crate::enums::poi::{BorderStyleEnum, FillPatternTypeEnum, HorizontalAlignmentEnum, VerticalAlignmentEnum};
use crate::{BooleanEnum, ExcelColor, ExcelDataFormat, StyleProperty, WriteCellStyle};

/// 内容单元格样式注解的全部参数及 Java 默认值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentStyle {
    data_format: i16, hidden: BooleanEnum, locked: BooleanEnum, quote_prefix: BooleanEnum,
    horizontal_alignment: HorizontalAlignmentEnum, wrapped: BooleanEnum,
    vertical_alignment: VerticalAlignmentEnum, rotation: i16, indent: i16,
    border_left: BorderStyleEnum, border_right: BorderStyleEnum, border_top: BorderStyleEnum,
    border_bottom: BorderStyleEnum, left_border_color: i16, right_border_color: i16,
    top_border_color: i16, bottom_border_color: i16, fill_pattern_type: FillPatternTypeEnum,
    fill_background_color: i16, fill_foreground_color: i16, shrink_to_fit: BooleanEnum,
}
impl Default for ContentStyle {
    fn default() -> Self { Self { data_format: -1, hidden: BooleanEnum::Default, locked: BooleanEnum::Default, quote_prefix: BooleanEnum::Default, horizontal_alignment: HorizontalAlignmentEnum::Default, wrapped: BooleanEnum::Default, vertical_alignment: VerticalAlignmentEnum::Default, rotation: -1, indent: -1, border_left: BorderStyleEnum::Default, border_right: BorderStyleEnum::Default, border_top: BorderStyleEnum::Default, border_bottom: BorderStyleEnum::Default, left_border_color: -1, right_border_color: -1, top_border_color: -1, bottom_border_color: -1, fill_pattern_type: FillPatternTypeEnum::Default, fill_background_color: -1, fill_foreground_color: -1, shrink_to_fit: BooleanEnum::Default } }
}
impl ContentStyle {
    /// 创建 Java 默认参数对象。
    #[must_use] pub fn new() -> Self { Self::default() }
    #[must_use] pub const fn data_format(&self) -> i16 { self.data_format }
    pub const fn set_data_format(&mut self, v: i16) { self.data_format = v; }
    #[must_use] pub const fn hidden(&self) -> BooleanEnum { self.hidden }
    pub const fn set_hidden(&mut self, v: BooleanEnum) { self.hidden = v; }
    #[must_use] pub const fn locked(&self) -> BooleanEnum { self.locked }
    pub const fn set_locked(&mut self, v: BooleanEnum) { self.locked = v; }
    #[must_use] pub const fn quote_prefix(&self) -> BooleanEnum { self.quote_prefix }
    pub const fn set_quote_prefix(&mut self, v: BooleanEnum) { self.quote_prefix = v; }
    #[must_use] pub const fn horizontal_alignment(&self) -> HorizontalAlignmentEnum { self.horizontal_alignment }
    pub const fn set_horizontal_alignment(&mut self, v: HorizontalAlignmentEnum) { self.horizontal_alignment = v; }
    #[must_use] pub const fn wrapped(&self) -> BooleanEnum { self.wrapped }
    pub const fn set_wrapped(&mut self, v: BooleanEnum) { self.wrapped = v; }
    #[must_use] pub const fn vertical_alignment(&self) -> VerticalAlignmentEnum { self.vertical_alignment }
    pub const fn set_vertical_alignment(&mut self, v: VerticalAlignmentEnum) { self.vertical_alignment = v; }
    #[must_use] pub const fn rotation(&self) -> i16 { self.rotation }
    pub const fn set_rotation(&mut self, v: i16) { self.rotation = v; }
    #[must_use] pub const fn indent(&self) -> i16 { self.indent }
    pub const fn set_indent(&mut self, v: i16) { self.indent = v; }
    #[must_use] pub const fn border_left(&self) -> BorderStyleEnum { self.border_left }
    pub const fn set_border_left(&mut self, v: BorderStyleEnum) { self.border_left = v; }
    #[must_use] pub const fn border_right(&self) -> BorderStyleEnum { self.border_right }
    pub const fn set_border_right(&mut self, v: BorderStyleEnum) { self.border_right = v; }
    #[must_use] pub const fn border_top(&self) -> BorderStyleEnum { self.border_top }
    pub const fn set_border_top(&mut self, v: BorderStyleEnum) { self.border_top = v; }
    #[must_use] pub const fn border_bottom(&self) -> BorderStyleEnum { self.border_bottom }
    pub const fn set_border_bottom(&mut self, v: BorderStyleEnum) { self.border_bottom = v; }
    #[must_use] pub const fn left_border_color(&self) -> i16 { self.left_border_color }
    pub const fn set_left_border_color(&mut self, v: i16) { self.left_border_color = v; }
    #[must_use] pub const fn right_border_color(&self) -> i16 { self.right_border_color }
    pub const fn set_right_border_color(&mut self, v: i16) { self.right_border_color = v; }
    #[must_use] pub const fn top_border_color(&self) -> i16 { self.top_border_color }
    pub const fn set_top_border_color(&mut self, v: i16) { self.top_border_color = v; }
    #[must_use] pub const fn bottom_border_color(&self) -> i16 { self.bottom_border_color }
    pub const fn set_bottom_border_color(&mut self, v: i16) { self.bottom_border_color = v; }
    #[must_use] pub const fn fill_pattern_type(&self) -> FillPatternTypeEnum { self.fill_pattern_type }
    pub const fn set_fill_pattern_type(&mut self, v: FillPatternTypeEnum) { self.fill_pattern_type = v; }
    #[must_use] pub const fn fill_background_color(&self) -> i16 { self.fill_background_color }
    pub const fn set_fill_background_color(&mut self, v: i16) { self.fill_background_color = v; }
    #[must_use] pub const fn fill_foreground_color(&self) -> i16 { self.fill_foreground_color }
    pub const fn set_fill_foreground_color(&mut self, v: i16) { self.fill_foreground_color = v; }
    #[must_use] pub const fn shrink_to_fit(&self) -> BooleanEnum { self.shrink_to_fit }
    pub const fn set_shrink_to_fit(&mut self, v: BooleanEnum) { self.shrink_to_fit = v; }
    /// 转换为写引擎样式，所有 Java sentinel 保持未指定状态。
    #[must_use]
    pub fn to_write_cell_style(self) -> WriteCellStyle {
        let color = |value: i16| u32::try_from(value).ok().map(ExcelColor::java_or_rgb);
        WriteCellStyle {
            hidden: self.hidden.value(), locked: self.locked.value(), quote_prefix: self.quote_prefix.value(),
            horizontal_alignment: self.horizontal_alignment.poi_horizontal_alignment(), wrapped: self.wrapped.value(),
            vertical_alignment: self.vertical_alignment.poi_vertical_alignment_enum(),
            rotation: (self.rotation >= 0).then_some(self.rotation), indent: u8::try_from(self.indent).ok(),
            border_left: self.border_left.poi_border_style(), border_right: self.border_right.poi_border_style(),
            border_top: self.border_top.poi_border_style(), border_bottom: self.border_bottom.poi_border_style(),
            left_border_color: color(self.left_border_color), right_border_color: color(self.right_border_color),
            top_border_color: color(self.top_border_color), bottom_border_color: color(self.bottom_border_color),
            fill_pattern: self.fill_pattern_type.poi_fill_pattern_type(),
            fill_background_color: color(self.fill_background_color), fill_foreground_color: color(self.fill_foreground_color),
            shrink_to_fit: self.shrink_to_fit.value(), data_format: u8::try_from(self.data_format).ok().map(ExcelDataFormat::Builtin), font: None,
        }
    }
    /// 转换为 Java `StyleProperty` 运行期镜像。
    #[must_use]
    pub fn to_property(self) -> StyleProperty {
        StyleProperty::from_write_cell_style(self.to_write_cell_style())
    }
}
