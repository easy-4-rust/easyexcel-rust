//! CsvCellStyle 的 STUB 方法集中文件。
//!
//! 包含 CSV 格式不支持的 Excel 单元格样式功能的 no-op 实现。
//! 对应 Java：com.alibaba.excel.metadata.csv.CsvCellStyle 中的 no-op 方法。

use crate::csv::CsvCellStyle;

/// CsvCellStyle 的 STUB 方法实现。
///
/// 这些方法对应 Java CsvCellStyle 中因 CSV 格式限制而无法实现的功能，
/// 保留 no-op 语义以维持 Java API 调用兼容性。
impl CsvCellStyle {
    // ─── 字体 (Font) ───

    /// CSV 不保存字体，Java 实现固定返回零。
    /// 对应 Java: CsvCellStyle#getFontIndex no-op
    #[must_use]
    pub const fn font_index(&self) -> usize {
        0
    }

    /// Java `getFontIndex()` 兼容别名。
    /// 对应 Java: CsvCellStyle#getFontIndex no-op
    pub const fn get_font_index(&self) -> usize {
        self.font_index()
    }
    /// Java `getFontIndexAsInt()` 兼容别名。
    /// 对应 Java: CsvCellStyle#getFontIndexAsInt no-op
    pub const fn get_font_index_as_int(&self) -> usize {
        self.font_index()
    }
    /// Java CSV 实现为空操作。
    /// 对应 Java: CsvCellStyle#setFont no-op
    pub const fn set_font(&mut self, _font: Option<()>) {}

    // ─── 单元格属性 (Cell Properties) ───

    /// CSV 不保存隐藏标志。
    /// 对应 Java: CsvCellStyle#isHidden no-op
    #[must_use]
    pub const fn hidden(&self) -> bool {
        false
    }
    /// 对应 Java: CsvCellStyle#getHidden no-op
    pub const fn get_hidden(&self) -> bool {
        self.hidden()
    }

    /// CSV 不保存锁定标志。
    /// 对应 Java: CsvCellStyle#isLocked no-op
    #[must_use]
    pub const fn locked(&self) -> bool {
        false
    }
    /// 对应 Java: CsvCellStyle#getLocked no-op
    pub const fn get_locked(&self) -> bool {
        self.locked()
    }

    /// CSV 不保存 quote-prefix 标志。
    /// 对应 Java: CsvCellStyle#getQuotePrefixed no-op
    #[must_use]
    pub const fn quote_prefixed(&self) -> bool {
        false
    }
    /// 对应 Java: CsvCellStyle#getQuotePrefixed no-op
    pub const fn get_quote_prefixed(&self) -> bool {
        self.quote_prefixed()
    }

    /// CSV 不保存换行标志。
    /// 对应 Java: CsvCellStyle#getWrapText no-op
    #[must_use]
    pub const fn wrap_text(&self) -> bool {
        false
    }
    /// 对应 Java: CsvCellStyle#getWrapText no-op
    pub const fn get_wrap_text(&self) -> bool {
        self.wrap_text()
    }

    /// CSV 不保存旋转角度。
    /// 对应 Java: CsvCellStyle#getRotation no-op
    #[must_use]
    pub const fn rotation(&self) -> i16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getRotation no-op
    pub const fn get_rotation(&self) -> i16 {
        self.rotation()
    }

    /// CSV 不保存缩进。
    /// 对应 Java: CsvCellStyle#getIndention no-op
    #[must_use]
    pub const fn indention(&self) -> i16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getIndention no-op
    pub const fn get_indention(&self) -> i16 {
        self.indention()
    }

    /// CSV 不保存 shrink-to-fit 标志。
    /// 对应 Java: CsvCellStyle#getShrinkToFit no-op
    #[must_use]
    pub const fn shrink_to_fit(&self) -> bool {
        false
    }
    /// 对应 Java: CsvCellStyle#getShrinkToFit no-op
    pub const fn get_shrink_to_fit(&self) -> bool {
        self.shrink_to_fit()
    }

    // ─── 对齐 (Alignment) ───

    /// CSV 不保存水平对齐；`None` 对应 Java 的 `null`。
    /// 对应 Java: CsvCellStyle#getAlignment no-op
    #[must_use]
    pub const fn alignment(&self) -> Option<u8> {
        None
    }
    /// 对应 Java: CsvCellStyle#getAlignment no-op
    pub const fn get_alignment(&self) -> Option<u8> {
        self.alignment()
    }

    /// CSV 不保存垂直对齐；`None` 对应 Java 的 `null`。
    /// 对应 Java: CsvCellStyle#getVerticalAlignment no-op
    #[must_use]
    pub const fn vertical_alignment(&self) -> Option<u8> {
        None
    }
    /// 对应 Java: CsvCellStyle#getVerticalAlignment no-op
    pub const fn get_vertical_alignment(&self) -> Option<u8> {
        self.vertical_alignment()
    }

    // ─── 边框 (Border) ───

    /// CSV 不保存边框；`None` 对应 Java 的 `null`。
    /// 对应 Java: CsvCellStyle#getBorderLeft no-op
    #[must_use]
    pub const fn border_left(&self) -> Option<u8> {
        None
    }
    /// 对应 Java: CsvCellStyle#getBorderLeft no-op
    pub const fn get_border_left(&self) -> Option<u8> {
        self.border_left()
    }

    /// CSV 不保存边框。
    /// 对应 Java: CsvCellStyle#getBorderRight no-op
    #[must_use]
    pub const fn border_right(&self) -> Option<u8> {
        None
    }
    /// 对应 Java: CsvCellStyle#getBorderRight no-op
    pub const fn get_border_right(&self) -> Option<u8> {
        self.border_right()
    }

    /// CSV 不保存边框。
    /// 对应 Java: CsvCellStyle#getBorderTop no-op
    #[must_use]
    pub const fn border_top(&self) -> Option<u8> {
        None
    }
    /// 对应 Java: CsvCellStyle#getBorderTop no-op
    pub const fn get_border_top(&self) -> Option<u8> {
        self.border_top()
    }

    /// CSV 不保存边框。
    /// 对应 Java: CsvCellStyle#getBorderBottom no-op
    #[must_use]
    pub const fn border_bottom(&self) -> Option<u8> {
        None
    }
    /// 对应 Java: CsvCellStyle#getBorderBottom no-op
    pub const fn get_border_bottom(&self) -> Option<u8> {
        self.border_bottom()
    }

    /// CSV 不保存边框颜色。
    /// 对应 Java: CsvCellStyle#getLeftBorderColor no-op
    #[must_use]
    pub const fn left_border_color(&self) -> u16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getLeftBorderColor no-op
    pub const fn get_left_border_color(&self) -> u16 {
        self.left_border_color()
    }

    /// CSV 不保存边框颜色。
    /// 对应 Java: CsvCellStyle#getRightBorderColor no-op
    #[must_use]
    pub const fn right_border_color(&self) -> u16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getRightBorderColor no-op
    pub const fn get_right_border_color(&self) -> u16 {
        self.right_border_color()
    }

    /// CSV 不保存边框颜色。
    /// 对应 Java: CsvCellStyle#getTopBorderColor no-op
    #[must_use]
    pub const fn top_border_color(&self) -> u16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getTopBorderColor no-op
    pub const fn get_top_border_color(&self) -> u16 {
        self.top_border_color()
    }

    /// CSV 不保存边框颜色。
    /// 对应 Java: CsvCellStyle#getBottomBorderColor no-op
    #[must_use]
    pub const fn bottom_border_color(&self) -> u16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getBottomBorderColor no-op
    pub const fn get_bottom_border_color(&self) -> u16 {
        self.bottom_border_color()
    }

    // ─── 填充 (Fill) ───

    /// CSV 不保存填充图案。
    /// 对应 Java: CsvCellStyle#getFillPattern no-op
    #[must_use]
    pub const fn fill_pattern(&self) -> Option<u8> {
        None
    }
    /// 对应 Java: CsvCellStyle#getFillPattern no-op
    pub const fn get_fill_pattern(&self) -> Option<u8> {
        self.fill_pattern()
    }

    /// CSV 不保存填充背景色。
    /// 对应 Java: CsvCellStyle#getFillBackgroundColor no-op
    #[must_use]
    pub const fn fill_background_color(&self) -> u16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getFillBackgroundColor no-op
    pub const fn get_fill_background_color(&self) -> u16 {
        self.fill_background_color()
    }
    /// 对应 Java: CsvCellStyle#getFillBackgroundColorColor no-op
    pub const fn get_fill_background_color_color(&self) -> Option<u16> {
        None
    }

    /// CSV 不保存填充前景色。
    /// 对应 Java: CsvCellStyle#getFillForegroundColor no-op
    #[must_use]
    pub const fn fill_foreground_color(&self) -> u16 {
        0
    }
    /// 对应 Java: CsvCellStyle#getFillForegroundColor no-op
    pub const fn get_fill_foreground_color(&self) -> u16 {
        self.fill_foreground_color()
    }
    /// 对应 Java: CsvCellStyle#getFillForegroundColorColor no-op
    pub const fn get_fill_foreground_color_color(&self) -> Option<u16> {
        None
    }

    // ─── Setter（no-op） ───

    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setHidden no-op
    pub const fn set_hidden(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setLocked no-op
    pub const fn set_locked(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setQuotePrefixed no-op
    pub const fn set_quote_prefixed(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setWrapText no-op
    pub const fn set_wrap_text(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setRotation no-op
    pub const fn set_rotation(&mut self, _value: i16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setIndention no-op
    pub const fn set_indention(&mut self, _value: i16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setShrinkToFit no-op
    pub const fn set_shrink_to_fit(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setAlignment no-op
    pub const fn set_alignment(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setVerticalAlignment no-op
    pub const fn set_vertical_alignment(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setBorderLeft no-op
    pub const fn set_border_left(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setBorderRight no-op
    pub const fn set_border_right(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setBorderTop no-op
    pub const fn set_border_top(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setBorderBottom no-op
    pub const fn set_border_bottom(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setLeftBorderColor no-op
    pub const fn set_left_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setRightBorderColor no-op
    pub const fn set_right_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setTopBorderColor no-op
    pub const fn set_top_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setBottomBorderColor no-op
    pub const fn set_bottom_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setFillPattern no-op
    pub const fn set_fill_pattern(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setFillBackgroundColor no-op
    pub const fn set_fill_background_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    /// 对应 Java: CsvCellStyle#setFillForegroundColor no-op
    pub const fn set_fill_foreground_color(&mut self, _value: u16) {}

    // ─── 克隆 (Clone) ───

    /// Java `cloneStyleFrom` 在 CSV 实现中是 no-op。
    /// 对应 Java: CsvCellStyle#cloneStyleFrom no-op
    pub const fn clone_style_from(&mut self, _source: &Self) {}
}

#[cfg(test)]
mod tests {
    use crate::csv::CsvCellStyle;

    #[test]
    fn font_stubs_return_fixed_values() {
        let style = CsvCellStyle::new(0);
        assert_eq!(style.font_index(), 0);
        assert_eq!(style.get_font_index(), 0);
        assert_eq!(style.get_font_index_as_int(), 0);
    }

    #[test]
    fn font_setter_is_noop() {
        let mut style = CsvCellStyle::new(0);
        style.set_font(None);
    }

    #[test]
    fn hidden_locked_quote_prefix_wrap_text_rotation_indention_shrink_to_fit() {
        let style = CsvCellStyle::new(0);
        assert!(!style.hidden());
        assert!(!style.get_hidden());
        assert!(!style.locked());
        assert!(!style.get_locked());
        assert!(!style.quote_prefixed());
        assert!(!style.get_quote_prefixed());
        assert!(!style.wrap_text());
        assert!(!style.get_wrap_text());
        assert_eq!(style.rotation(), 0);
        assert_eq!(style.get_rotation(), 0);
        assert_eq!(style.indention(), 0);
        assert_eq!(style.get_indention(), 0);
        assert!(!style.shrink_to_fit());
        assert!(!style.get_shrink_to_fit());
    }

    #[test]
    fn setters_are_noop() {
        let mut style = CsvCellStyle::new(0);
        style.set_hidden(true);
        style.set_locked(true);
        style.set_quote_prefixed(true);
        style.set_wrap_text(true);
        style.set_rotation(45);
        style.set_indention(3);
        style.set_shrink_to_fit(true);
        // 值不变
        assert!(!style.hidden());
    }

    #[test]
    fn alignment_stubs_return_none() {
        let style = CsvCellStyle::new(0);
        assert!(style.alignment().is_none());
        assert!(style.get_alignment().is_none());
        assert!(style.vertical_alignment().is_none());
        assert!(style.get_vertical_alignment().is_none());
    }

    #[test]
    fn alignment_setters_are_noop() {
        let mut style = CsvCellStyle::new(0);
        style.set_alignment(Some(1));
        style.set_vertical_alignment(Some(2));
    }

    #[test]
    fn border_stubs_return_none_or_zero() {
        let style = CsvCellStyle::new(0);
        assert!(style.border_left().is_none());
        assert!(style.border_right().is_none());
        assert!(style.border_top().is_none());
        assert!(style.border_bottom().is_none());
        assert_eq!(style.left_border_color(), 0);
        assert_eq!(style.right_border_color(), 0);
        assert_eq!(style.top_border_color(), 0);
        assert_eq!(style.bottom_border_color(), 0);
        assert_eq!(style.get_border_left(), None);
        assert_eq!(style.get_border_right(), None);
        assert_eq!(style.get_border_top(), None);
        assert_eq!(style.get_border_bottom(), None);
        assert_eq!(style.get_left_border_color(), 0);
        assert_eq!(style.get_right_border_color(), 0);
        assert_eq!(style.get_top_border_color(), 0);
        assert_eq!(style.get_bottom_border_color(), 0);
    }

    #[test]
    fn border_setters_are_noop() {
        let mut style = CsvCellStyle::new(0);
        style.set_border_left(Some(1));
        style.set_border_right(Some(1));
        style.set_border_top(Some(1));
        style.set_border_bottom(Some(1));
        style.set_left_border_color(1);
        style.set_right_border_color(1);
        style.set_top_border_color(1);
        style.set_bottom_border_color(1);
    }

    #[test]
    fn fill_stubs_return_none_or_zero() {
        let style = CsvCellStyle::new(0);
        assert!(style.fill_pattern().is_none());
        assert!(style.get_fill_pattern().is_none());
        assert_eq!(style.fill_background_color(), 0);
        assert_eq!(style.get_fill_background_color(), 0);
        assert!(style.get_fill_background_color_color().is_none());
        assert_eq!(style.fill_foreground_color(), 0);
        assert_eq!(style.get_fill_foreground_color(), 0);
        assert!(style.get_fill_foreground_color_color().is_none());
    }

    #[test]
    fn fill_setters_are_noop() {
        let mut style = CsvCellStyle::new(0);
        style.set_fill_pattern(Some(1));
        style.set_fill_background_color(1);
        style.set_fill_foreground_color(1);
    }

    #[test]
    fn clone_style_from_is_noop() {
        let source = CsvCellStyle::new(5);
        let mut target = CsvCellStyle::new(0);
        target.clone_style_from(&source);
        assert_eq!(target.index(), 0);
    }
}
