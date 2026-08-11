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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_java_sentinel_values() {
        let cs = ContentStyle::default();
        assert_eq!(cs.data_format(), -1);
        assert_eq!(cs.hidden(), BooleanEnum::Default);
        assert_eq!(cs.locked(), BooleanEnum::Default);
        assert_eq!(cs.quote_prefix(), BooleanEnum::Default);
        assert_eq!(cs.horizontal_alignment(), HorizontalAlignmentEnum::Default);
        assert_eq!(cs.wrapped(), BooleanEnum::Default);
        assert_eq!(cs.vertical_alignment(), VerticalAlignmentEnum::Default);
        assert_eq!(cs.rotation(), -1);
        assert_eq!(cs.indent(), -1);
        assert_eq!(cs.border_left(), BorderStyleEnum::Default);
        assert_eq!(cs.border_right(), BorderStyleEnum::Default);
        assert_eq!(cs.border_top(), BorderStyleEnum::Default);
        assert_eq!(cs.border_bottom(), BorderStyleEnum::Default);
        assert_eq!(cs.left_border_color(), -1);
        assert_eq!(cs.right_border_color(), -1);
        assert_eq!(cs.top_border_color(), -1);
        assert_eq!(cs.bottom_border_color(), -1);
        assert_eq!(cs.fill_pattern_type(), FillPatternTypeEnum::Default);
        assert_eq!(cs.fill_background_color(), -1);
        assert_eq!(cs.fill_foreground_color(), -1);
        assert_eq!(cs.shrink_to_fit(), BooleanEnum::Default);
    }

    #[test]
    fn new_returns_default() {
        let cs = ContentStyle::new();
        assert_eq!(cs, ContentStyle::default());
    }

    #[test]
    fn setters_and_getters_roundtrip() {
        let mut cs = ContentStyle::new();
        cs.set_data_format(5);
        assert_eq!(cs.data_format(), 5);

        cs.set_hidden(BooleanEnum::True);
        assert_eq!(cs.hidden(), BooleanEnum::True);

        cs.set_locked(BooleanEnum::False);
        assert_eq!(cs.locked(), BooleanEnum::False);

        cs.set_quote_prefix(BooleanEnum::True);
        assert_eq!(cs.quote_prefix(), BooleanEnum::True);

        cs.set_horizontal_alignment(HorizontalAlignmentEnum::Center);
        assert_eq!(cs.horizontal_alignment(), HorizontalAlignmentEnum::Center);

        cs.set_wrapped(BooleanEnum::True);
        assert_eq!(cs.wrapped(), BooleanEnum::True);

        cs.set_vertical_alignment(VerticalAlignmentEnum::Top);
        assert_eq!(cs.vertical_alignment(), VerticalAlignmentEnum::Top);

        cs.set_rotation(45);
        assert_eq!(cs.rotation(), 45);

        cs.set_indent(2);
        assert_eq!(cs.indent(), 2);

        cs.set_border_left(BorderStyleEnum::Thin);
        assert_eq!(cs.border_left(), BorderStyleEnum::Thin);

        cs.set_border_right(BorderStyleEnum::Medium);
        assert_eq!(cs.border_right(), BorderStyleEnum::Medium);

        cs.set_border_top(BorderStyleEnum::Dashed);
        assert_eq!(cs.border_top(), BorderStyleEnum::Dashed);

        cs.set_border_bottom(BorderStyleEnum::Double);
        assert_eq!(cs.border_bottom(), BorderStyleEnum::Double);

        cs.set_left_border_color(10);
        assert_eq!(cs.left_border_color(), 10);

        cs.set_right_border_color(20);
        assert_eq!(cs.right_border_color(), 20);

        cs.set_top_border_color(30);
        assert_eq!(cs.top_border_color(), 30);

        cs.set_bottom_border_color(40);
        assert_eq!(cs.bottom_border_color(), 40);

        cs.set_fill_pattern_type(FillPatternTypeEnum::SolidForeground);
        assert_eq!(cs.fill_pattern_type(), FillPatternTypeEnum::SolidForeground);

        cs.set_fill_background_color(50);
        assert_eq!(cs.fill_background_color(), 50);

        cs.set_fill_foreground_color(60);
        assert_eq!(cs.fill_foreground_color(), 60);

        cs.set_shrink_to_fit(BooleanEnum::True);
        assert_eq!(cs.shrink_to_fit(), BooleanEnum::True);
    }

    #[test]
    fn to_write_cell_style_defaults_produce_none_fields() {
        let cs = ContentStyle::default();
        let wcs = cs.to_write_cell_style();
        // 默认 sentinel 值应映射为 None
        assert!(wcs.hidden.is_none());
        assert!(wcs.locked.is_none());
        assert!(wcs.quote_prefix.is_none());
        assert!(wcs.horizontal_alignment.is_none());
        assert!(wcs.wrapped.is_none());
        assert!(wcs.vertical_alignment.is_none());
        assert!(wcs.rotation.is_none());
        assert!(wcs.indent.is_none());
        assert!(wcs.border_left.is_none());
        assert!(wcs.border_right.is_none());
        assert!(wcs.border_top.is_none());
        assert!(wcs.border_bottom.is_none());
        assert!(wcs.fill_pattern.is_none());
        assert!(wcs.shrink_to_fit.is_none());
    }

    #[test]
    fn to_write_cell_style_with_values() {
        let mut cs = ContentStyle::new();
        cs.set_hidden(BooleanEnum::True);
        cs.set_locked(BooleanEnum::False);
        cs.set_horizontal_alignment(HorizontalAlignmentEnum::Left);
        cs.set_vertical_alignment(VerticalAlignmentEnum::Bottom);
        cs.set_rotation(15);
        cs.set_indent(3);
        cs.set_border_left(BorderStyleEnum::Thin);
        cs.set_fill_pattern_type(FillPatternTypeEnum::SolidForeground);
        cs.set_shrink_to_fit(BooleanEnum::True);
        cs.set_data_format(1);

        let wcs = cs.to_write_cell_style();
        assert_eq!(wcs.hidden, Some(true));
        assert_eq!(wcs.locked, Some(false));
        assert_eq!(wcs.horizontal_alignment, Some(crate::ExcelHorizontalAlignment::Left));
        assert_eq!(wcs.vertical_alignment, Some(crate::ExcelVerticalAlignment::Bottom));
        assert_eq!(wcs.rotation, Some(15));
        assert_eq!(wcs.indent, Some(3));
        assert_eq!(wcs.border_left, Some(crate::ExcelBorderStyle::Thin));
        assert_eq!(wcs.fill_pattern, Some(crate::ExcelFillPattern::Solid));
        assert_eq!(wcs.shrink_to_fit, Some(true));
    }

    #[test]
    fn to_write_cell_style_negative_rotation_is_none() {
        let mut cs = ContentStyle::new();
        cs.set_rotation(-1);
        let wcs = cs.to_write_cell_style();
        assert!(wcs.rotation.is_none());
    }

    #[test]
    fn to_property_produces_non_default() {
        let mut cs = ContentStyle::new();
        cs.set_hidden(BooleanEnum::True);
        let prop = cs.to_property();
        assert_eq!(prop.get_hidden(), Some(true));
    }

    #[test]
    fn copy_clone_eq() {
        let cs = ContentStyle::new();
        let b = cs;
        let c = cs.clone();
        assert_eq!(cs, b);
        assert_eq!(cs, c);
    }

    #[test]
    fn debug_contains_struct_name() {
        let cs = ContentStyle::new();
        let text = format!("{cs:?}");
        assert!(text.contains("ContentStyle"));
    }
}
