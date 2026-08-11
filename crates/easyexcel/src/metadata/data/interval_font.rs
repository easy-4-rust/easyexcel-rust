//! 对应 Java：`com.alibaba.excel.metadata.data.RichTextStringData.IntervalFont`.

use crate::core::write_font::WriteFont;

/// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData.IntervalFont。 One Java `RichTextStringData.IntervalFont` range using UTF-16 indices.
///
/// Java keeps `Integer` for both indices; Rust uses `usize` to match
/// `std::str::encode_utf16` and to align with how the rest of the
/// `easyexcel-rust` workspace indexes strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct IntervalFont {
    start_index: usize,
    end_index: usize,
    write_font: WriteFont,
}

impl IntervalFont {
    /// Creates a half-open font range `[start_index, end_index)`. (Java inner `IntervalFont(int, int, WriteFont)`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData.IntervalFont。
    pub const fn new(start_index: usize, end_index: usize, write_font: WriteFont) -> Self {
        Self {
            start_index,
            end_index,
            write_font,
        }
    }

    /// Returns the inclusive UTF-16 start index. (Java `getStartIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData.IntervalFont。
    pub const fn start_index(&self) -> usize {
        self.start_index
    }

    /// Java `getStartIndex` 兼容别名。
    #[must_use]
    pub const fn get_start_index(&self) -> usize {
        self.start_index()
    }
    /// 设置 UTF-16 起始下标。
    pub const fn set_start_index(&mut self, value: usize) {
        self.start_index = value;
    }

    /// Returns the exclusive UTF-16 end index. (Java `getEndIndex()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData.IntervalFont。
    pub const fn end_index(&self) -> usize {
        self.end_index
    }

    /// Java `getEndIndex` 兼容别名。
    #[must_use]
    pub const fn get_end_index(&self) -> usize {
        self.end_index()
    }
    /// 设置 UTF-16 结束下标。
    pub const fn set_end_index(&mut self, value: usize) {
        self.end_index = value;
    }

    /// Returns the interval font. (Java `getWriteFont()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.data.RichTextStringData.IntervalFont。
    pub const fn write_font(&self) -> &WriteFont {
        &self.write_font
    }

    /// Java `getWriteFont` 兼容别名。
    #[must_use]
    pub const fn get_write_font(&self) -> &WriteFont {
        self.write_font()
    }
    /// 设置区间字体。
    pub fn set_write_font(&mut self, value: WriteFont) {
        self.write_font = value;
    }
}
