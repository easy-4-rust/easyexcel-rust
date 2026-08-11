//! 对应 Java：`com.alibaba.excel.annotation.write.style.HeadStyle`。

use super::ContentStyle;
use crate::enums::poi::{
    BorderStyleEnum, FillPatternTypeEnum, HorizontalAlignmentEnum, VerticalAlignmentEnum,
};
use crate::{BooleanEnum, StyleProperty, WriteCellStyle};

/// 表头样式注解；保持独立 Java 类型并复用完全相同的参数语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeadStyle {
    values: ContentStyle,
}
impl HeadStyle {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    #[must_use]
    pub const fn data_format(&self) -> i16 {
        self.values.data_format()
    }
    pub const fn set_data_format(&mut self, v: i16) {
        self.values.set_data_format(v);
    }
    #[must_use]
    pub const fn hidden(&self) -> BooleanEnum {
        self.values.hidden()
    }
    pub const fn set_hidden(&mut self, v: BooleanEnum) {
        self.values.set_hidden(v);
    }
    #[must_use]
    pub const fn locked(&self) -> BooleanEnum {
        self.values.locked()
    }
    pub const fn set_locked(&mut self, v: BooleanEnum) {
        self.values.set_locked(v);
    }
    #[must_use]
    pub const fn quote_prefix(&self) -> BooleanEnum {
        self.values.quote_prefix()
    }
    pub const fn set_quote_prefix(&mut self, v: BooleanEnum) {
        self.values.set_quote_prefix(v);
    }
    #[must_use]
    pub const fn horizontal_alignment(&self) -> HorizontalAlignmentEnum {
        self.values.horizontal_alignment()
    }
    pub const fn set_horizontal_alignment(&mut self, v: HorizontalAlignmentEnum) {
        self.values.set_horizontal_alignment(v);
    }
    #[must_use]
    pub const fn wrapped(&self) -> BooleanEnum {
        self.values.wrapped()
    }
    pub const fn set_wrapped(&mut self, v: BooleanEnum) {
        self.values.set_wrapped(v);
    }
    #[must_use]
    pub const fn vertical_alignment(&self) -> VerticalAlignmentEnum {
        self.values.vertical_alignment()
    }
    pub const fn set_vertical_alignment(&mut self, v: VerticalAlignmentEnum) {
        self.values.set_vertical_alignment(v);
    }
    #[must_use]
    pub const fn rotation(&self) -> i16 {
        self.values.rotation()
    }
    pub const fn set_rotation(&mut self, v: i16) {
        self.values.set_rotation(v);
    }
    #[must_use]
    pub const fn indent(&self) -> i16 {
        self.values.indent()
    }
    pub const fn set_indent(&mut self, v: i16) {
        self.values.set_indent(v);
    }
    #[must_use]
    pub const fn border_left(&self) -> BorderStyleEnum {
        self.values.border_left()
    }
    pub const fn set_border_left(&mut self, v: BorderStyleEnum) {
        self.values.set_border_left(v);
    }
    #[must_use]
    pub const fn border_right(&self) -> BorderStyleEnum {
        self.values.border_right()
    }
    pub const fn set_border_right(&mut self, v: BorderStyleEnum) {
        self.values.set_border_right(v);
    }
    #[must_use]
    pub const fn border_top(&self) -> BorderStyleEnum {
        self.values.border_top()
    }
    pub const fn set_border_top(&mut self, v: BorderStyleEnum) {
        self.values.set_border_top(v);
    }
    #[must_use]
    pub const fn border_bottom(&self) -> BorderStyleEnum {
        self.values.border_bottom()
    }
    pub const fn set_border_bottom(&mut self, v: BorderStyleEnum) {
        self.values.set_border_bottom(v);
    }
    #[must_use]
    pub const fn left_border_color(&self) -> i16 {
        self.values.left_border_color()
    }
    pub const fn set_left_border_color(&mut self, v: i16) {
        self.values.set_left_border_color(v);
    }
    #[must_use]
    pub const fn right_border_color(&self) -> i16 {
        self.values.right_border_color()
    }
    pub const fn set_right_border_color(&mut self, v: i16) {
        self.values.set_right_border_color(v);
    }
    #[must_use]
    pub const fn top_border_color(&self) -> i16 {
        self.values.top_border_color()
    }
    pub const fn set_top_border_color(&mut self, v: i16) {
        self.values.set_top_border_color(v);
    }
    #[must_use]
    pub const fn bottom_border_color(&self) -> i16 {
        self.values.bottom_border_color()
    }
    pub const fn set_bottom_border_color(&mut self, v: i16) {
        self.values.set_bottom_border_color(v);
    }
    #[must_use]
    pub const fn fill_pattern_type(&self) -> FillPatternTypeEnum {
        self.values.fill_pattern_type()
    }
    pub const fn set_fill_pattern_type(&mut self, v: FillPatternTypeEnum) {
        self.values.set_fill_pattern_type(v);
    }
    #[must_use]
    pub const fn fill_background_color(&self) -> i16 {
        self.values.fill_background_color()
    }
    pub const fn set_fill_background_color(&mut self, v: i16) {
        self.values.set_fill_background_color(v);
    }
    #[must_use]
    pub const fn fill_foreground_color(&self) -> i16 {
        self.values.fill_foreground_color()
    }
    pub const fn set_fill_foreground_color(&mut self, v: i16) {
        self.values.set_fill_foreground_color(v);
    }
    #[must_use]
    pub const fn shrink_to_fit(&self) -> BooleanEnum {
        self.values.shrink_to_fit()
    }
    pub const fn set_shrink_to_fit(&mut self, v: BooleanEnum) {
        self.values.set_shrink_to_fit(v);
    }
    #[must_use]
    pub fn to_write_cell_style(self) -> WriteCellStyle {
        self.values.to_write_cell_style()
    }
    #[must_use]
    pub fn to_property(self) -> StyleProperty {
        self.values.to_property()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_delegates_to_content_style_default() {
        let hs = HeadStyle::default();
        let cs = ContentStyle::default();
        assert_eq!(hs.data_format(), cs.data_format());
        assert_eq!(hs.hidden(), cs.hidden());
        assert_eq!(hs.locked(), cs.locked());
        assert_eq!(hs.quote_prefix(), cs.quote_prefix());
        assert_eq!(hs.horizontal_alignment(), cs.horizontal_alignment());
        assert_eq!(hs.wrapped(), cs.wrapped());
        assert_eq!(hs.vertical_alignment(), cs.vertical_alignment());
        assert_eq!(hs.rotation(), cs.rotation());
        assert_eq!(hs.indent(), cs.indent());
        assert_eq!(hs.border_left(), cs.border_left());
        assert_eq!(hs.border_right(), cs.border_right());
        assert_eq!(hs.border_top(), cs.border_top());
        assert_eq!(hs.border_bottom(), cs.border_bottom());
        assert_eq!(hs.fill_pattern_type(), cs.fill_pattern_type());
        assert_eq!(hs.shrink_to_fit(), cs.shrink_to_fit());
    }

    #[test]
    fn new_returns_default() {
        let hs = HeadStyle::new();
        assert_eq!(hs, HeadStyle::default());
    }

    #[test]
    fn setters_and_getters_roundtrip() {
        let mut hs = HeadStyle::new();
        hs.set_data_format(10);
        assert_eq!(hs.data_format(), 10);

        hs.set_hidden(BooleanEnum::True);
        assert_eq!(hs.hidden(), BooleanEnum::True);

        hs.set_locked(BooleanEnum::False);
        assert_eq!(hs.locked(), BooleanEnum::False);

        hs.set_quote_prefix(BooleanEnum::True);
        assert_eq!(hs.quote_prefix(), BooleanEnum::True);

        hs.set_horizontal_alignment(HorizontalAlignmentEnum::Right);
        assert_eq!(hs.horizontal_alignment(), HorizontalAlignmentEnum::Right);

        hs.set_wrapped(BooleanEnum::True);
        assert_eq!(hs.wrapped(), BooleanEnum::True);

        hs.set_vertical_alignment(VerticalAlignmentEnum::Center);
        assert_eq!(hs.vertical_alignment(), VerticalAlignmentEnum::Center);

        hs.set_rotation(90);
        assert_eq!(hs.rotation(), 90);

        hs.set_indent(4);
        assert_eq!(hs.indent(), 4);

        hs.set_border_left(BorderStyleEnum::Thick);
        assert_eq!(hs.border_left(), BorderStyleEnum::Thick);

        hs.set_border_right(BorderStyleEnum::Hair);
        assert_eq!(hs.border_right(), BorderStyleEnum::Hair);

        hs.set_border_top(BorderStyleEnum::Dotted);
        assert_eq!(hs.border_top(), BorderStyleEnum::Dotted);

        hs.set_border_bottom(BorderStyleEnum::SlantedDashDot);
        assert_eq!(hs.border_bottom(), BorderStyleEnum::SlantedDashDot);

        hs.set_left_border_color(100);
        assert_eq!(hs.left_border_color(), 100);

        hs.set_right_border_color(200);
        assert_eq!(hs.right_border_color(), 200);

        hs.set_top_border_color(300);
        assert_eq!(hs.top_border_color(), 300);

        hs.set_bottom_border_color(400);
        assert_eq!(hs.bottom_border_color(), 400);

        hs.set_fill_pattern_type(FillPatternTypeEnum::FineDots);
        assert_eq!(hs.fill_pattern_type(), FillPatternTypeEnum::FineDots);

        hs.set_fill_background_color(500);
        assert_eq!(hs.fill_background_color(), 500);

        hs.set_fill_foreground_color(600);
        assert_eq!(hs.fill_foreground_color(), 600);

        hs.set_shrink_to_fit(BooleanEnum::True);
        assert_eq!(hs.shrink_to_fit(), BooleanEnum::True);
    }

    #[test]
    fn to_write_cell_style_converts_values() {
        let mut hs = HeadStyle::new();
        hs.set_hidden(BooleanEnum::True);
        hs.set_border_left(BorderStyleEnum::Thin);
        let wcs = hs.to_write_cell_style();
        assert_eq!(wcs.hidden, Some(true));
        assert_eq!(wcs.border_left, Some(crate::ExcelBorderStyle::Thin));
    }

    #[test]
    fn to_property_converts() {
        let mut hs = HeadStyle::new();
        hs.set_hidden(BooleanEnum::True);
        let prop = hs.to_property();
        assert_eq!(prop.get_hidden(), Some(true));
    }

    #[test]
    fn copy_clone_eq() {
        let hs = HeadStyle::new();
        let b = hs;
        let c = hs.clone();
        assert_eq!(hs, b);
        assert_eq!(hs, c);
    }

    #[test]
    fn debug_contains_struct_name() {
        let hs = HeadStyle::new();
        let text = format!("{hs:?}");
        assert!(text.contains("HeadStyle"));
    }
}
