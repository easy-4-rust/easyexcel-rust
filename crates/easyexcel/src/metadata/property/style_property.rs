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
        Self {
            write_cell_style: WriteCellStyle::new(),
        }
    }

    /// Java 默认对象别名。
    #[must_use]
    pub const fn empty() -> Self {
        Self::new()
    }

    /// 从注解期轻量样式创建运行期属性。
    #[must_use]
    pub fn from_cell_style(cell_style: ExcelCellStyle) -> Self {
        Self {
            write_cell_style: cell_style.into(),
        }
    }

    /// 从完整运行期样式创建属性。
    #[must_use]
    pub const fn from_write_cell_style(write_cell_style: WriteCellStyle) -> Self {
        Self { write_cell_style }
    }

    /// 返回完整运行期样式。
    #[must_use]
    pub const fn write_cell_style(&self) -> &WriteCellStyle {
        &self.write_cell_style
    }

    /// 消费属性并返回完整运行期样式。
    #[must_use]
    pub fn into_write_cell_style(self) -> WriteCellStyle {
        self.write_cell_style
    }

    /// Java `getDataFormatData`。
    #[must_use]
    pub const fn get_data_format_data(&self) -> Option<ExcelDataFormat> {
        self.write_cell_style.data_format
    }
    /// Java `setDataFormatData`。
    pub const fn set_data_format_data(&mut self, value: Option<ExcelDataFormat>) {
        self.write_cell_style.data_format = value;
    }
    /// Java `getWriteFont`。
    #[must_use]
    pub const fn get_write_font(&self) -> Option<&WriteFont> {
        self.write_cell_style.font.as_ref()
    }
    /// Java `setWriteFont`。
    pub fn set_write_font(&mut self, value: Option<WriteFont>) {
        self.write_cell_style.font = value;
    }
    /// Java `getHidden`。
    #[must_use]
    pub const fn get_hidden(&self) -> Option<bool> {
        self.write_cell_style.hidden
    }
    /// Java `setHidden`。
    pub const fn set_hidden(&mut self, value: Option<bool>) {
        self.write_cell_style.hidden = value;
    }
    /// Java `getLocked`。
    #[must_use]
    pub const fn get_locked(&self) -> Option<bool> {
        self.write_cell_style.locked
    }
    /// Java `setLocked`。
    pub const fn set_locked(&mut self, value: Option<bool>) {
        self.write_cell_style.locked = value;
    }
    /// Java `getQuotePrefix`。
    #[must_use]
    pub const fn get_quote_prefix(&self) -> Option<bool> {
        self.write_cell_style.quote_prefix
    }
    /// Java `setQuotePrefix`。
    pub const fn set_quote_prefix(&mut self, value: Option<bool>) {
        self.write_cell_style.quote_prefix = value;
    }
    /// Java `getHorizontalAlignment`。
    #[must_use]
    pub const fn get_horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> {
        self.write_cell_style.horizontal_alignment
    }
    /// Java `setHorizontalAlignment`。
    pub const fn set_horizontal_alignment(&mut self, value: Option<ExcelHorizontalAlignment>) {
        self.write_cell_style.horizontal_alignment = value;
    }
    /// Java `getWrapped`。
    #[must_use]
    pub const fn get_wrapped(&self) -> Option<bool> {
        self.write_cell_style.wrapped
    }
    /// Java `setWrapped`。
    pub const fn set_wrapped(&mut self, value: Option<bool>) {
        self.write_cell_style.wrapped = value;
    }
    /// Java `getVerticalAlignment`。
    #[must_use]
    pub const fn get_vertical_alignment(&self) -> Option<ExcelVerticalAlignment> {
        self.write_cell_style.vertical_alignment
    }
    /// Java `setVerticalAlignment`。
    pub const fn set_vertical_alignment(&mut self, value: Option<ExcelVerticalAlignment>) {
        self.write_cell_style.vertical_alignment = value;
    }
    /// Java `getRotation`。
    #[must_use]
    pub const fn get_rotation(&self) -> Option<i16> {
        self.write_cell_style.rotation
    }
    /// Java `setRotation`。
    pub const fn set_rotation(&mut self, value: Option<i16>) {
        self.write_cell_style.rotation = value;
    }
    /// Java `getIndent`。
    #[must_use]
    pub const fn get_indent(&self) -> Option<u8> {
        self.write_cell_style.indent
    }
    /// Java `setIndent`。
    pub const fn set_indent(&mut self, value: Option<u8>) {
        self.write_cell_style.indent = value;
    }
    /// Java `getBorderLeft`。
    #[must_use]
    pub const fn get_border_left(&self) -> Option<ExcelBorderStyle> {
        self.write_cell_style.border_left
    }
    /// Java `setBorderLeft`。
    pub const fn set_border_left(&mut self, value: Option<ExcelBorderStyle>) {
        self.write_cell_style.border_left = value;
    }
    /// Java `getBorderRight`。
    #[must_use]
    pub const fn get_border_right(&self) -> Option<ExcelBorderStyle> {
        self.write_cell_style.border_right
    }
    /// Java `setBorderRight`。
    pub const fn set_border_right(&mut self, value: Option<ExcelBorderStyle>) {
        self.write_cell_style.border_right = value;
    }
    /// Java `getBorderTop`。
    #[must_use]
    pub const fn get_border_top(&self) -> Option<ExcelBorderStyle> {
        self.write_cell_style.border_top
    }
    /// Java `setBorderTop`。
    pub const fn set_border_top(&mut self, value: Option<ExcelBorderStyle>) {
        self.write_cell_style.border_top = value;
    }
    /// Java `getBorderBottom`。
    #[must_use]
    pub const fn get_border_bottom(&self) -> Option<ExcelBorderStyle> {
        self.write_cell_style.border_bottom
    }
    /// Java `setBorderBottom`。
    pub const fn set_border_bottom(&mut self, value: Option<ExcelBorderStyle>) {
        self.write_cell_style.border_bottom = value;
    }
    /// Java `getLeftBorderColor`。
    #[must_use]
    pub const fn get_left_border_color(&self) -> Option<ExcelColor> {
        self.write_cell_style.left_border_color
    }
    /// Java `setLeftBorderColor`。
    pub const fn set_left_border_color(&mut self, value: Option<ExcelColor>) {
        self.write_cell_style.left_border_color = value;
    }
    /// Java `getRightBorderColor`。
    #[must_use]
    pub const fn get_right_border_color(&self) -> Option<ExcelColor> {
        self.write_cell_style.right_border_color
    }
    /// Java `setRightBorderColor`。
    pub const fn set_right_border_color(&mut self, value: Option<ExcelColor>) {
        self.write_cell_style.right_border_color = value;
    }
    /// Java `getTopBorderColor`。
    #[must_use]
    pub const fn get_top_border_color(&self) -> Option<ExcelColor> {
        self.write_cell_style.top_border_color
    }
    /// Java `setTopBorderColor`。
    pub const fn set_top_border_color(&mut self, value: Option<ExcelColor>) {
        self.write_cell_style.top_border_color = value;
    }
    /// Java `getBottomBorderColor`。
    #[must_use]
    pub const fn get_bottom_border_color(&self) -> Option<ExcelColor> {
        self.write_cell_style.bottom_border_color
    }
    /// Java `setBottomBorderColor`。
    pub const fn set_bottom_border_color(&mut self, value: Option<ExcelColor>) {
        self.write_cell_style.bottom_border_color = value;
    }
    /// Java `getFillPatternType`。
    #[must_use]
    pub const fn get_fill_pattern_type(&self) -> Option<ExcelFillPattern> {
        self.write_cell_style.fill_pattern
    }
    /// Java `setFillPatternType`。
    pub const fn set_fill_pattern_type(&mut self, value: Option<ExcelFillPattern>) {
        self.write_cell_style.fill_pattern = value;
    }
    /// Java `getFillBackgroundColor`。
    #[must_use]
    pub const fn get_fill_background_color(&self) -> Option<ExcelColor> {
        self.write_cell_style.fill_background_color
    }
    /// Java `setFillBackgroundColor`。
    pub const fn set_fill_background_color(&mut self, value: Option<ExcelColor>) {
        self.write_cell_style.fill_background_color = value;
    }
    /// Java `getFillForegroundColor`。
    #[must_use]
    pub const fn get_fill_foreground_color(&self) -> Option<ExcelColor> {
        self.write_cell_style.fill_foreground_color
    }
    /// Java `setFillForegroundColor`。
    pub const fn set_fill_foreground_color(&mut self, value: Option<ExcelColor>) {
        self.write_cell_style.fill_foreground_color = value;
    }
    /// Java `getShrinkToFit`。
    #[must_use]
    pub const fn get_shrink_to_fit(&self) -> Option<bool> {
        self.write_cell_style.shrink_to_fit
    }
    /// Java `setShrinkToFit`。
    pub const fn set_shrink_to_fit(&mut self, value: Option<bool>) {
        self.write_cell_style.shrink_to_fit = value;
    }

    /// Rust 风格隐藏标志 getter。
    #[must_use]
    pub const fn hidden(&self) -> Option<bool> {
        self.get_hidden()
    }
    /// Rust 风格锁定标志 getter。
    #[must_use]
    pub const fn locked(&self) -> Option<bool> {
        self.get_locked()
    }
    /// Rust 风格 quote-prefix getter。
    #[must_use]
    pub const fn quote_prefix(&self) -> Option<bool> {
        self.get_quote_prefix()
    }
    /// Rust 风格水平对齐 getter。
    #[must_use]
    pub const fn horizontal_alignment(&self) -> Option<ExcelHorizontalAlignment> {
        self.get_horizontal_alignment()
    }
    /// Rust 风格换行 getter。
    #[must_use]
    pub const fn wrapped(&self) -> Option<bool> {
        self.get_wrapped()
    }
    /// Rust 风格垂直对齐 getter。
    #[must_use]
    pub const fn vertical_alignment(&self) -> Option<ExcelVerticalAlignment> {
        self.get_vertical_alignment()
    }
    /// Rust 风格旋转角 getter。
    #[must_use]
    pub const fn rotation(&self) -> Option<i16> {
        self.get_rotation()
    }
    /// Rust 风格缩进 getter。
    #[must_use]
    pub const fn indent(&self) -> Option<u8> {
        self.get_indent()
    }
    /// Rust 风格左边框 getter。
    #[must_use]
    pub const fn border_left(&self) -> Option<ExcelBorderStyle> {
        self.get_border_left()
    }
    /// Rust 风格右边框 getter。
    #[must_use]
    pub const fn border_right(&self) -> Option<ExcelBorderStyle> {
        self.get_border_right()
    }
    /// Rust 风格上边框 getter。
    #[must_use]
    pub const fn border_top(&self) -> Option<ExcelBorderStyle> {
        self.get_border_top()
    }
    /// Rust 风格下边框 getter。
    #[must_use]
    pub const fn border_bottom(&self) -> Option<ExcelBorderStyle> {
        self.get_border_bottom()
    }
    /// Rust 风格左边框颜色 getter。
    #[must_use]
    pub const fn left_border_color(&self) -> Option<ExcelColor> {
        self.get_left_border_color()
    }
    /// Rust 风格右边框颜色 getter。
    #[must_use]
    pub const fn right_border_color(&self) -> Option<ExcelColor> {
        self.get_right_border_color()
    }
    /// Rust 风格上边框颜色 getter。
    #[must_use]
    pub const fn top_border_color(&self) -> Option<ExcelColor> {
        self.get_top_border_color()
    }
    /// Rust 风格下边框颜色 getter。
    #[must_use]
    pub const fn bottom_border_color(&self) -> Option<ExcelColor> {
        self.get_bottom_border_color()
    }
    /// Rust 风格填充图案 getter。
    #[must_use]
    pub const fn fill_pattern_type(&self) -> Option<ExcelFillPattern> {
        self.get_fill_pattern_type()
    }
    /// Rust 风格填充背景色 getter。
    #[must_use]
    pub const fn fill_background_color(&self) -> Option<ExcelColor> {
        self.get_fill_background_color()
    }
    /// Rust 风格填充前景色 getter。
    #[must_use]
    pub const fn fill_foreground_color(&self) -> Option<ExcelColor> {
        self.get_fill_foreground_color()
    }
    /// Rust 风格 shrink-to-fit getter。
    #[must_use]
    pub const fn shrink_to_fit(&self) -> Option<bool> {
        self.get_shrink_to_fit()
    }
    /// Rust 风格数字格式 getter。
    #[must_use]
    pub const fn data_format_data(&self) -> Option<ExcelDataFormat> {
        self.get_data_format_data()
    }
    /// Rust 风格字体 getter。
    #[must_use]
    pub const fn write_font(&self) -> Option<&WriteFont> {
        self.get_write_font()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_all_none() {
        // 对应 Java：StyleProperty 无参构造器
        let style = StyleProperty::new();
        assert!(style.get_data_format_data().is_none());
        assert!(style.get_write_font().is_none());
        assert!(style.get_hidden().is_none());
        assert!(style.get_locked().is_none());
        assert!(style.get_quote_prefix().is_none());
        assert!(style.get_horizontal_alignment().is_none());
        assert!(style.get_wrapped().is_none());
        assert!(style.get_vertical_alignment().is_none());
        assert!(style.get_rotation().is_none());
        assert!(style.get_indent().is_none());
        assert!(style.get_border_left().is_none());
        assert!(style.get_border_right().is_none());
        assert!(style.get_border_top().is_none());
        assert!(style.get_border_bottom().is_none());
        assert!(style.get_left_border_color().is_none());
        assert!(style.get_right_border_color().is_none());
        assert!(style.get_top_border_color().is_none());
        assert!(style.get_bottom_border_color().is_none());
        assert!(style.get_fill_pattern_type().is_none());
        assert!(style.get_fill_background_color().is_none());
        assert!(style.get_fill_foreground_color().is_none());
        assert!(style.get_shrink_to_fit().is_none());
    }

    #[test]
    fn empty_alias_for_new() {
        // 对应 Java：empty() 等价于 new()
        assert_eq!(StyleProperty::new(), StyleProperty::empty());
    }

    #[test]
    fn default_trait_returns_empty() {
        // 对应 Java：Default 派生
        let style = StyleProperty::default();
        assert_eq!(style, StyleProperty::new());
    }

    #[test]
    fn from_cell_style_creates_property() {
        // 对应 Java：fromCellStyle 转换
        let cell_style = ExcelCellStyle::default();
        let style = StyleProperty::from_cell_style(cell_style);
        let _ = style;
    }

    #[test]
    fn from_write_cell_style_creates_property() {
        // 对应 Java：fromWriteCellStyle 转换
        let write_style = WriteCellStyle::new();
        let style = StyleProperty::from_write_cell_style(write_style);
        let _ = style;
    }

    #[test]
    fn write_cell_style_accessor() {
        // 对应 Java：getWriteCellStyle
        let style = StyleProperty::new();
        let _wcs: &WriteCellStyle = style.write_cell_style();
    }

    #[test]
    fn into_write_cell_style_consumes() {
        // 对应 Java：intoWriteCellStyle
        let style = StyleProperty::new();
        let _wcs: WriteCellStyle = style.into_write_cell_style();
    }

    #[test]
    fn hidden_setter_and_getter() {
        // 对应 Java：hidden getter/setter
        let mut style = StyleProperty::new();
        assert!(style.hidden().is_none());
        style.set_hidden(Some(true));
        assert_eq!(style.hidden(), Some(true));
        assert_eq!(style.get_hidden(), Some(true));
    }

    #[test]
    fn locked_setter_and_getter() {
        // 对应 Java：locked getter/setter
        let mut style = StyleProperty::new();
        style.set_locked(Some(true));
        assert_eq!(style.locked(), Some(true));
    }

    #[test]
    fn quote_prefix_setter_and_getter() {
        // 对应 Java：quotePrefix getter/setter
        let mut style = StyleProperty::new();
        style.set_quote_prefix(Some(false));
        assert_eq!(style.quote_prefix(), Some(false));
    }

    #[test]
    fn horizontal_alignment_setter_and_getter() {
        // 对应 Java：horizontalAlignment getter/setter
        let mut style = StyleProperty::new();
        style.set_horizontal_alignment(Some(ExcelHorizontalAlignment::Center));
        assert_eq!(
            style.horizontal_alignment(),
            Some(ExcelHorizontalAlignment::Center)
        );
    }

    #[test]
    fn wrapped_setter_and_getter() {
        // 对应 Java：wrapped getter/setter
        let mut style = StyleProperty::new();
        style.set_wrapped(Some(true));
        assert_eq!(style.wrapped(), Some(true));
    }

    #[test]
    fn vertical_alignment_setter_and_getter() {
        // 对应 Java：verticalAlignment getter/setter
        let mut style = StyleProperty::new();
        style.set_vertical_alignment(Some(ExcelVerticalAlignment::Center));
        assert_eq!(
            style.vertical_alignment(),
            Some(ExcelVerticalAlignment::Center)
        );
    }

    #[test]
    fn rotation_setter_and_getter() {
        // 对应 Java：rotation getter/setter
        let mut style = StyleProperty::new();
        style.set_rotation(Some(45));
        assert_eq!(style.rotation(), Some(45));
    }

    #[test]
    fn indent_setter_and_getter() {
        // 对应 Java：indent getter/setter
        let mut style = StyleProperty::new();
        style.set_indent(Some(2));
        assert_eq!(style.indent(), Some(2));
    }

    #[test]
    fn border_left_setter_and_getter() {
        // 对应 Java：borderLeft getter/setter
        let mut style = StyleProperty::new();
        style.set_border_left(Some(ExcelBorderStyle::Thin));
        assert_eq!(style.border_left(), Some(ExcelBorderStyle::Thin));
    }

    #[test]
    fn border_right_setter_and_getter() {
        // 对应 Java：borderRight getter/setter
        let mut style = StyleProperty::new();
        style.set_border_right(Some(ExcelBorderStyle::Medium));
        assert_eq!(style.border_right(), Some(ExcelBorderStyle::Medium));
    }

    #[test]
    fn border_top_setter_and_getter() {
        // 对应 Java：borderTop getter/setter
        let mut style = StyleProperty::new();
        style.set_border_top(Some(ExcelBorderStyle::Thick));
        assert_eq!(style.border_top(), Some(ExcelBorderStyle::Thick));
    }

    #[test]
    fn border_bottom_setter_and_getter() {
        // 对应 Java：borderBottom getter/setter
        let mut style = StyleProperty::new();
        style.set_border_bottom(Some(ExcelBorderStyle::Hair));
        assert_eq!(style.border_bottom(), Some(ExcelBorderStyle::Hair));
    }

    #[test]
    fn border_color_setters_and_getters() {
        // 对应 Java：borderColor getter/setter
        let mut style = StyleProperty::new();
        style.set_left_border_color(Some(ExcelColor::Rgb(0xFF0000)));
        assert_eq!(style.left_border_color(), Some(ExcelColor::Rgb(0xFF0000)));
        style.set_right_border_color(Some(ExcelColor::Rgb(0x00FF00)));
        assert_eq!(style.right_border_color(), Some(ExcelColor::Rgb(0x00FF00)));
        style.set_top_border_color(Some(ExcelColor::Rgb(0x0000FF)));
        assert_eq!(style.top_border_color(), Some(ExcelColor::Rgb(0x0000FF)));
        style.set_bottom_border_color(Some(ExcelColor::Indexed(1)));
        assert_eq!(style.bottom_border_color(), Some(ExcelColor::Indexed(1)));
    }

    #[test]
    fn fill_setters_and_getters() {
        // 对应 Java：fill getter/setter
        let mut style = StyleProperty::new();
        style.set_fill_pattern_type(Some(ExcelFillPattern::Solid));
        assert_eq!(style.fill_pattern_type(), Some(ExcelFillPattern::Solid));
        style.set_fill_background_color(Some(ExcelColor::Rgb(0xCCCCCC)));
        assert_eq!(
            style.fill_background_color(),
            Some(ExcelColor::Rgb(0xCCCCCC))
        );
        style.set_fill_foreground_color(Some(ExcelColor::Rgb(0x333333)));
        assert_eq!(
            style.fill_foreground_color(),
            Some(ExcelColor::Rgb(0x333333))
        );
    }

    #[test]
    fn shrink_to_fit_setter_and_getter() {
        // 对应 Java：shrinkToFit getter/setter
        let mut style = StyleProperty::new();
        style.set_shrink_to_fit(Some(true));
        assert_eq!(style.shrink_to_fit(), Some(true));
    }

    #[test]
    fn data_format_data_setter_and_getter() {
        // 对应 Java：dataFormatData getter/setter
        let mut style = StyleProperty::new();
        assert!(style.data_format_data().is_none());
        style.set_data_format_data(None);
        assert!(style.data_format_data().is_none());
    }

    #[test]
    fn write_font_setter_and_getter() {
        // 对应 Java：writeFont getter/setter
        let mut style = StyleProperty::new();
        assert!(style.write_font().is_none());
        style.set_write_font(Some(WriteFont::new()));
        assert!(style.write_font().is_some());
        style.set_write_font(None);
        assert!(style.write_font().is_none());
    }

    #[test]
    fn clone_produces_equal_instance() {
        // 对应 Java：clone
        let mut style = StyleProperty::new();
        style.set_hidden(Some(true));
        style.set_locked(Some(true));
        let cloned = style.clone();
        assert_eq!(style, cloned);
    }

    #[test]
    fn hash_consistency() {
        // 对应 Java：相同内容哈希一致
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut a = StyleProperty::new();
        a.set_hidden(Some(true));
        let mut b = StyleProperty::new();
        b.set_hidden(Some(true));
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        b.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}
