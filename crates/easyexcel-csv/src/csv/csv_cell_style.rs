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
    pub const fn get_index(&self) -> i16 {
        self.index()
    }

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
    pub fn get_data_format(&self) -> i16 {
        self.data_format()
    }

    /// 返回数字格式文本。
    #[must_use]
    pub fn data_format_string(&self) -> Option<&str> {
        self.data_format_data
            .as_ref()
            .and_then(|data| data.format.as_deref())
    }

    /// Java `getDataFormatString()` 兼容别名。
    pub fn get_data_format_string(&self) -> Option<&str> {
        self.data_format_string()
    }

    /// Java `getDataFormatData()` 兼容别名。
    pub const fn get_data_format_data(&self) -> Option<&DataFormatData> {
        self.data_format_data()
    }
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
