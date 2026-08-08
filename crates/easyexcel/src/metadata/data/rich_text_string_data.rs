//! 对应 Java：`com.alibaba.excel.metadata.data.RichTextStringData`.

use crate::core::cell_value::CellValue;
use crate::core::convert_context::ConvertContext;
use crate::core::excel_error::ExcelError;
use crate::core::from_excel_cell::FromExcelCell;
use crate::core::interval_font::IntervalFont;
use crate::core::into_excel_cell::IntoExcelCell;
use crate::core::write_font::WriteFont;

/// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。 Java `RichTextStringData` equivalent.
///
/// Java exposes `textString`, `writeFont`, `intervalFontList` via Lombok
/// accessors. Rust preserves the same fields and offers builder-style
/// `apply_font` / `apply_font_range` setters matching the Java semantics.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RichTextStringData {
    text_string: String,
    write_font: Option<WriteFont>,
    interval_font_list: Vec<IntervalFont>,
}

impl RichTextStringData {
    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。 Creates rich-text metadata for a string. (Java `RichTextStringData(String)`)
    #[must_use]
    pub fn new(text_string: impl Into<String>) -> Self {
        Self {
            text_string: text_string.into(),
            write_font: None,
            interval_font_list: Vec::new(),
        }
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。 Applies a font to the entire string. (Java `applyFont(WriteFont)`)
    #[must_use]
    pub fn apply_font(mut self, write_font: WriteFont) -> Self {
        self.write_font = Some(write_font);
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。 Applies a font to a half-open UTF-16 character range. (Java `applyFont(int, int, WriteFont)`)
    #[must_use]
    pub fn apply_font_range(
        mut self,
        start_index: usize,
        end_index: usize,
        write_font: WriteFont,
    ) -> Self {
        self.interval_font_list
            .push(IntervalFont::new(start_index, end_index, write_font));
        self
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。 Replaces all interval font entries.
    #[must_use]
    pub fn interval_font_list(mut self, value: impl IntoIterator<Item = IntervalFont>) -> Self {
        self.interval_font_list = value.into_iter().collect();
        self
    }

    /// 设置富文本原始字符串。
    pub fn set_text_string(&mut self, value: impl Into<String>) { self.text_string = value.into(); }

    /// 设置整串字体。
    pub fn set_write_font(&mut self, value: Option<WriteFont>) { self.write_font = value; }

    /// 替换全部区间字体。
    pub fn set_interval_font_list(&mut self, value: impl IntoIterator<Item = IntervalFont>) {
        self.interval_font_list = value.into_iter().collect();
    }

    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。 Returns the underlying text. (Java `getTextString()`)
    #[must_use]
    pub fn text_string(&self) -> &str {
        &self.text_string
    }

    /// Java `getTextString` 兼容别名。
    #[must_use]
    pub fn get_text_string(&self) -> &str { self.text_string() }

    /// Returns the optional whole-string font. (Java `getWriteFont()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。
    pub const fn write_font(&self) -> Option<&WriteFont> {
        self.write_font.as_ref()
    }

    /// Java `getWriteFont` 兼容别名。
    #[must_use]
    pub const fn get_write_font(&self) -> Option<&WriteFont> { self.write_font() }

    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData。 Returns interval fonts in application order. (Java `getIntervalFontList()`)
    #[must_use]
    pub fn interval_fonts(&self) -> &[IntervalFont] {
        &self.interval_font_list
    }

    /// Java `getIntervalFontList` 兼容别名。
    #[must_use]
    pub fn get_interval_font_list(&self) -> &[IntervalFont] { self.interval_fonts() }
}

impl IntoExcelCell for RichTextStringData {
    fn to_excel_cell(&self, _context: &ConvertContext) -> Result<CellValue, ExcelError> {
        Ok(CellValue::RichText(self.clone()))
    }
}

impl FromExcelCell for RichTextStringData {
    fn from_excel_cell(
        cell: Option<&CellValue>,
        _context: &ConvertContext,
    ) -> Result<Self, ExcelError> {
        Ok(match cell {
            Some(CellValue::RichText(value)) => value.clone(),
            _ => Self::new(cell.map_or_else(String::new, CellValue::as_text)),
        })
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn interval_font_list_replaces_entries() {
        // 对应 Java：RichTextStringData 的 intervalFontList 整体替换
        let font = WriteFont::new().bold(true);
        let value = RichTextStringData::new("rich").interval_font_list(vec![
            IntervalFont::new(0, 2, font.clone()),
            IntervalFont::new(2, 4, font),
        ]);
        assert_eq!(value.interval_fonts().len(), 2);
        assert_eq!(value.interval_fonts()[0].start_index(), 0);
        assert_eq!(value.text_string(), "rich");
        assert!(value.write_font().is_none());
    }

    #[test]
    fn apply_font_and_range_builders() {
        // 对应 Java：applyFont / applyFont 区间
        let font = WriteFont::new().italic(true);
        let value = RichTextStringData::new("hello")
            .apply_font(font.clone())
            .apply_font_range(1, 3, font);
        assert!(value.write_font().is_some());
        assert_eq!(value.interval_fonts().len(), 1);
    }

    #[test]
    fn from_excel_cell_preserves_rich_text_metadata() {
        let value =
            RichTextStringData::new("hello").apply_font_range(1, 4, WriteFont::new().bold(true));
        let cell = CellValue::RichText(value.clone());
        let context = ConvertContext {
            sheet_name: "Sheet1".to_owned(),
            row_index: 0,
            column_index: Some(0),
            field: "value",
            format: None,
            date_time_format: None,
            number_format: None,
            use_1904_windowing: false,
        };
        let decoded = RichTextStringData::from_excel_cell(Some(&cell), &context)
            .expect("rich text conversion");
        assert_eq!(decoded, value);
    }
}
