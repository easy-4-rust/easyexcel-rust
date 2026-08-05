//! 工作簿无关的数据格式元数据。
//!
//! 语义对应 Java：`com.alibaba.excel.metadata.data.DataFormatData`，供
//! XLS、XLSX、CSV 与 EasyExcel 门面共同使用。

use std::borrow::Cow;

/// 单元格数据格式索引与自定义格式文本。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DataFormatData {
    /// 工作簿内的格式索引。
    pub index: Option<i16>,
    /// 自定义格式文本。
    pub format: Option<String>,
}

impl DataFormatData {
    /// 创建空的数据格式元数据。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回格式索引。
    #[must_use]
    pub const fn index(&self) -> Option<i16> {
        self.index
    }

    /// 设置格式索引。
    pub fn set_index(&mut self, index: Option<i16>) {
        self.index = index;
    }

    /// 返回自定义格式文本。
    #[must_use]
    pub fn format(&self) -> Option<&str> {
        self.format.as_deref()
    }

    /// 设置自定义格式文本。
    pub fn set_format(&mut self, format: impl Into<Option<String>>) {
        self.format = format.into();
    }

    /// 将来源中的非空字段合并到目标对象。
    pub fn merge(source: Option<&Self>, target: Option<&mut Self>) {
        let (Some(source), Some(target)) = (source, target) else {
            return;
        };
        if let Some(index) = source.index {
            target.index = Some(index);
        }
        if let Some(format) = source.format.as_ref().filter(|value| !value.trim().is_empty()) {
            target.format = Some(format.clone());
        }
    }

    /// 返回独立副本。
    #[must_use]
    pub fn clone_data(&self) -> Self {
        self.clone()
    }

    /// 以借用优先的形式返回格式文本。
    #[must_use]
    pub fn format_cow(&self) -> Cow<'_, str> {
        match &self.format {
            Some(value) => Cow::Borrowed(value.as_str()),
            None => Cow::Borrowed(""),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_overwrites_non_empty_fields() {
        let source = DataFormatData {
            index: Some(1),
            format: Some("0.00".to_owned()),
        };
        let mut target = DataFormatData::new();
        DataFormatData::merge(Some(&source), Some(&mut target));
        assert_eq!(target.index, Some(1));
        assert_eq!(target.format.as_deref(), Some("0.00"));
    }

    #[test]
    fn accessors_merge_and_clone_preserve_java_semantics() {
        let mut data = DataFormatData::new();
        data.set_index(Some(5));
        data.set_format(Some("0.00%".to_owned()));
        assert_eq!(data.index(), Some(5));
        assert_eq!(data.format(), Some("0.00%"));
        assert_eq!(data.clone_data(), data);
        assert_eq!(data.format_cow(), Cow::Borrowed("0.00%"));

        let source = DataFormatData {
            index: None,
            format: Some("   ".to_owned()),
        };
        DataFormatData::merge(Some(&source), Some(&mut data));
        assert_eq!(data.index(), Some(5));
        assert_eq!(data.format(), Some("0.00%"));
    }
}
