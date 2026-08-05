//! CSV 工作簿的数据格式注册表。
//!
//! 语义对应 Java：`com.alibaba.excel.metadata.csv.CsvDataFormat`。

use std::collections::HashMap;

use easyexcel_format::{MIN_CUSTOM_DATA_FORMAT_INDEX, switch_builtin_formats};

/// 工作簿局部的 CSV 数据格式注册表。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CsvDataFormat {
    custom_indexes: HashMap<String, i16>,
    custom_formats: Vec<String>,
}

impl CsvDataFormat {
    /// 创建空的自定义格式注册表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回内建格式或工作簿局部格式的索引。
    pub fn get_format_index(&mut self, format: &str) -> i16 {
        if let Some(index) = switch_builtin_formats()
            .iter()
            .position(|candidate| candidate.is_some_and(|candidate| candidate == format))
        {
            return i16::try_from(index).unwrap_or(i16::MAX);
        }
        if let Some(index) = self.custom_indexes.get(format) {
            return *index;
        }
        let index = usize::from(MIN_CUSTOM_DATA_FORMAT_INDEX) + self.custom_formats.len();
        let index = i16::try_from(index).unwrap_or(i16::MAX);
        self.custom_formats.push(format.to_owned());
        self.custom_indexes.insert(format.to_owned(), index);
        index
    }

    /// 按索引解析内建格式或工作簿局部格式文本。
    #[must_use]
    pub fn get_format(&self, index: i16) -> Option<&str> {
        let index = usize::try_from(index).ok()?;
        if index < usize::from(MIN_CUSTOM_DATA_FORMAT_INDEX) {
            return switch_builtin_formats().get(index).copied().flatten();
        }
        self.custom_formats
            .get(index - usize::from(MIN_CUSTOM_DATA_FORMAT_INDEX))
            .map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reuses_builtin_and_custom_indexes() {
        let mut formats = CsvDataFormat::new();
        assert_eq!(formats.get_format_index("0.00"), 2);
        let custom = formats.get_format_index("yyyy-mm-dd hh:mm:ss.000");
        assert_eq!(custom, 82);
        assert_eq!(formats.get_format_index("yyyy-mm-dd hh:mm:ss.000"), custom);
        assert_eq!(formats.get_format(custom), Some("yyyy-mm-dd hh:mm:ss.000"));
    }

    #[test]
    fn resolves_builtin_unknown_and_registered_formats() {
        let mut formats = CsvDataFormat::new();
        assert_eq!(formats.get_format(0), Some("General"));
        assert_eq!(formats.get_format(14), Some("yyyy/m/d"));
        assert_eq!(formats.get_format(99), None);
        assert_eq!(formats.get_format(-1), None);

        let index = formats.get_format_index("my-format");
        assert_eq!(index, 82);
        assert_eq!(formats.get_format(index), Some("my-format"));
    }
}
