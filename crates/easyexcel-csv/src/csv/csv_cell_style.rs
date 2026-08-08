//! CSV 单元格样式模型。
//!
//! 语义对应 Java：`com.alibaba.excel.metadata.csv.CsvCellStyle`。

use easyexcel_model::DataFormatData;

/// 对应 Java：com.alibaba.excel.metadata.csv.CsvCellStyle。 CSV 单元格样式；仅保留格式索引与格式文本。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvCellStyle {
    index: i16,
    data_format_data: Option<DataFormatData>,
}

impl CsvCellStyle {
    /// 按工作簿局部索引创建样式。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvCellStyle。
    pub const fn new(index: i16) -> Self {
        Self {
            index,
            data_format_data: None,
        }
    }

    /// 返回工作簿局部样式索引。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvCellStyle。
    pub const fn index(&self) -> i16 {
        self.index
    }

    /// Java `getIndex()` 兼容别名。
    pub const fn get_index(&self) -> i16 { self.index() }

    /// 设置工作簿局部样式索引，语义对应 Java Lombok `setIndex`。
    pub const fn set_index(&mut self, index: i16) {
        self.index = index;
    }

    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvCellStyle。 设置数字格式索引。
    pub fn set_data_format(&mut self, format: i16) {
        self.data_format_data
            .get_or_insert_with(DataFormatData::default)
            .index = Some(format);
    }

    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvCellStyle。 设置自定义数据格式文本。
    pub fn set_data_format_string(&mut self, format: impl Into<String>) {
        self.data_format_data
            .get_or_insert_with(DataFormatData::default)
            .format = Some(format.into());
    }

    /// 返回数据格式元数据。
    #[must_use]
    /// 对应 Java：com.alibaba.excel.metadata.csv.CsvCellStyle。
    pub const fn data_format_data(&self) -> Option<&DataFormatData> {
        self.data_format_data.as_ref()
    }

    /// 替换数据格式元数据，语义对应 Java Lombok `setDataFormatData`。
    pub fn set_data_format_data(&mut self, data: Option<DataFormatData>) {
        self.data_format_data = data;
    }

    /// 返回数字格式索引；未设置时与 Java 一样返回零。
    #[must_use]
    pub fn data_format(&self) -> i16 {
        self.data_format_data
            .as_ref()
            .and_then(|data| data.index)
            .unwrap_or(0)
    }

    /// Java `getDataFormat()` 兼容别名。
    pub fn get_data_format(&self) -> i16 { self.data_format() }

    /// 返回数字格式文本。
    #[must_use]
    pub fn data_format_string(&self) -> Option<&str> {
        self.data_format_data
            .as_ref()
            .and_then(|data| data.format.as_deref())
    }

    /// Java `getDataFormatString()` 兼容别名。
    pub fn get_data_format_string(&self) -> Option<&str> { self.data_format_string() }

    /// CSV 不保存字体，Java 实现固定返回零。
    #[must_use]
    pub const fn font_index(&self) -> usize {
        0
    }

    /// Java `getFontIndex()` 兼容别名。
    pub const fn get_font_index(&self) -> usize { self.font_index() }
    /// Java `getFontIndexAsInt()` 兼容别名。
    pub const fn get_font_index_as_int(&self) -> usize { self.font_index() }
    /// Java CSV 实现为空操作。
    pub const fn set_font(&mut self, _font: Option<()>) {}

    /// CSV 不保存隐藏标志。
    #[must_use]
    pub const fn hidden(&self) -> bool {
        false
    }
    pub const fn get_hidden(&self) -> bool { self.hidden() }

    /// CSV 不保存锁定标志。
    #[must_use]
    pub const fn locked(&self) -> bool {
        false
    }
    pub const fn get_locked(&self) -> bool { self.locked() }

    /// CSV 不保存 quote-prefix 标志。
    #[must_use]
    pub const fn quote_prefixed(&self) -> bool {
        false
    }
    pub const fn get_quote_prefixed(&self) -> bool { self.quote_prefixed() }

    /// CSV 不保存换行标志。
    #[must_use]
    pub const fn wrap_text(&self) -> bool {
        false
    }
    pub const fn get_wrap_text(&self) -> bool { self.wrap_text() }

    /// CSV 不保存旋转角度。
    #[must_use]
    pub const fn rotation(&self) -> i16 {
        0
    }
    pub const fn get_rotation(&self) -> i16 { self.rotation() }

    /// CSV 不保存缩进。
    #[must_use]
    pub const fn indention(&self) -> i16 {
        0
    }
    pub const fn get_indention(&self) -> i16 { self.indention() }

    /// CSV 不保存 shrink-to-fit 标志。
    #[must_use]
    pub const fn shrink_to_fit(&self) -> bool {
        false
    }
    pub const fn get_shrink_to_fit(&self) -> bool { self.shrink_to_fit() }

    /// CSV 不保存水平对齐；`None` 对应 Java 的 `null`。
    #[must_use]
    pub const fn alignment(&self) -> Option<u8> {
        None
    }
    pub const fn get_alignment(&self) -> Option<u8> { self.alignment() }

    /// CSV 不保存垂直对齐；`None` 对应 Java 的 `null`。
    #[must_use]
    pub const fn vertical_alignment(&self) -> Option<u8> {
        None
    }
    pub const fn get_vertical_alignment(&self) -> Option<u8> { self.vertical_alignment() }

    /// CSV 不保存边框；`None` 对应 Java 的 `null`。
    #[must_use]
    pub const fn border_left(&self) -> Option<u8> { None }
    pub const fn get_border_left(&self) -> Option<u8> { self.border_left() }

    /// CSV 不保存边框。
    #[must_use]
    pub const fn border_right(&self) -> Option<u8> { None }
    pub const fn get_border_right(&self) -> Option<u8> { self.border_right() }

    /// CSV 不保存边框。
    #[must_use]
    pub const fn border_top(&self) -> Option<u8> { None }
    pub const fn get_border_top(&self) -> Option<u8> { self.border_top() }

    /// CSV 不保存边框。
    #[must_use]
    pub const fn border_bottom(&self) -> Option<u8> { None }
    pub const fn get_border_bottom(&self) -> Option<u8> { self.border_bottom() }

    /// CSV 不保存边框颜色。
    #[must_use]
    pub const fn left_border_color(&self) -> u16 { 0 }
    pub const fn get_left_border_color(&self) -> u16 { self.left_border_color() }

    /// CSV 不保存边框颜色。
    #[must_use]
    pub const fn right_border_color(&self) -> u16 { 0 }
    pub const fn get_right_border_color(&self) -> u16 { self.right_border_color() }

    /// CSV 不保存边框颜色。
    #[must_use]
    pub const fn top_border_color(&self) -> u16 { 0 }
    pub const fn get_top_border_color(&self) -> u16 { self.top_border_color() }

    /// CSV 不保存边框颜色。
    #[must_use]
    pub const fn bottom_border_color(&self) -> u16 { 0 }
    pub const fn get_bottom_border_color(&self) -> u16 { self.bottom_border_color() }

    /// CSV 不保存填充图案。
    #[must_use]
    pub const fn fill_pattern(&self) -> Option<u8> { None }
    pub const fn get_fill_pattern(&self) -> Option<u8> { self.fill_pattern() }

    /// CSV 不保存填充背景色。
    #[must_use]
    pub const fn fill_background_color(&self) -> u16 { 0 }
    pub const fn get_fill_background_color(&self) -> u16 { self.fill_background_color() }
    pub const fn get_fill_background_color_color(&self) -> Option<u16> { None }

    /// CSV 不保存填充前景色。
    #[must_use]
    pub const fn fill_foreground_color(&self) -> u16 { 0 }
    pub const fn get_fill_foreground_color(&self) -> u16 { self.fill_foreground_color() }
    pub const fn get_fill_foreground_color_color(&self) -> Option<u16> { None }

    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_hidden(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_locked(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_quote_prefixed(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_wrap_text(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_rotation(&mut self, _value: i16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_indention(&mut self, _value: i16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_shrink_to_fit(&mut self, _value: bool) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_alignment(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_vertical_alignment(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_border_left(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_border_right(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_border_top(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_border_bottom(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_left_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_right_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_top_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_bottom_border_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_fill_pattern(&mut self, _value: Option<u8>) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_fill_background_color(&mut self, _value: u16) {}
    /// Java CSV 实现的非数据格式样式 setter 均为 no-op。
    pub const fn set_fill_foreground_color(&mut self, _value: u16) {}

    /// Java `cloneStyleFrom` 在 CSV 实现中是 no-op。
    pub const fn clone_style_from(&mut self, _source: &Self) {}

    /// Java `getDataFormatData()` 兼容别名。
    pub const fn get_data_format_data(&self) -> Option<&DataFormatData> { self.data_format_data() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_index_and_text_share_one_metadata_object() {
        let mut style = CsvCellStyle::new(7);
        style.set_data_format(5);
        style.set_data_format_string("0.00");
        let data = style.data_format_data().expect("format metadata");
        assert_eq!(style.index(), 7);
        assert_eq!(data.index, Some(5));
        assert_eq!(data.format.as_deref(), Some("0.00"));
    }

    #[test]
    fn setters_create_and_reuse_format_metadata() {
        let mut style = CsvCellStyle::new(0);
        assert!(style.data_format_data().is_none());
        style.set_data_format_string("0.00");
        style.set_data_format(3);
        let data = style.data_format_data().expect("format metadata");
        assert_eq!(data.index, Some(3));
        assert_eq!(data.format.as_deref(), Some("0.00"));
    }
}
