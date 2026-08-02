//! Mirrors Java `com.alibaba.excel.metadata.csv.CsvCellStyle`.

use crate::metadata::data::DataFormatData;

/// CSV cell-style metadata.
///
/// Like Java, only the style index and data format affect CSV rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvCellStyle {
    index: i16,
    data_format_data: Option<DataFormatData>,
}

impl CsvCellStyle {
    /// Creates a style with its workbook-local index.
    #[must_use]
    pub const fn new(index: i16) -> Self {
        Self {
            index,
            data_format_data: None,
        }
    }

    /// Returns the workbook-local style index.
    #[must_use]
    pub const fn index(&self) -> i16 {
        self.index
    }

    /// Sets the numeric data-format index.
    pub fn set_data_format(&mut self, format: i16) {
        self.data_format_data
            .get_or_insert_with(DataFormatData::default)
            .index = Some(format);
    }

    /// Sets an owned data-format string.
    pub fn set_data_format_string(&mut self, format: impl Into<String>) {
        self.data_format_data
            .get_or_insert_with(DataFormatData::default)
            .format = Some(format.into());
    }

    /// Returns the nested data-format metadata.
    #[must_use]
    pub const fn data_format_data(&self) -> Option<&DataFormatData> {
        self.data_format_data.as_ref()
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_and_index_accessor() {
        // 对应 Java：CsvCellStyle 索引
        let style = CsvCellStyle::new(7);
        assert_eq!(style.index(), 7);
        assert!(style.data_format_data().is_none());
    }

    #[test]
    fn set_data_format_and_string_create_metadata() {
        // 对应 Java：setDataFormat / setDataFormatString
        let mut style = CsvCellStyle::new(0);
        style.set_data_format(5);
        let data_format = style.data_format_data().expect("created");
        assert_eq!(data_format.index, Some(5));
        assert_eq!(data_format.format, None);

        // 再次设置仅更新索引，不重建元数据
        style.set_data_format(6);
        assert_eq!(style.data_format_data().expect("kept").index, Some(6));
    }

    #[test]
    fn set_data_format_string_stores_owned_format() {
        // 对应 Java：自定义格式串
        let mut style = CsvCellStyle::new(0);
        style.set_data_format_string("0.00");
        let data_format = style.data_format_data().expect("created");
        assert_eq!(data_format.format.as_deref(), Some("0.00"));

        // 先设置字符串再设置索引，二者共存
        style.set_data_format(3);
        let merged = style.data_format_data().expect("kept");
        assert_eq!(merged.index, Some(3));
        assert_eq!(merged.format.as_deref(), Some("0.00"));
    }
}
