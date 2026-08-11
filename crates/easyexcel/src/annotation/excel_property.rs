//! 对应 Java：`com.alibaba.excel.annotation.ExcelProperty`。

/// 字段到 Excel 列的声明式映射。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcelProperty {
    value: Vec<String>,
    index: i32,
    order: i32,
    converter: String,
    format: String,
}

impl Default for ExcelProperty {
    fn default() -> Self {
        Self {
            value: vec![String::new()],
            index: -1,
            order: i32::MAX,
            converter: "com.alibaba.excel.converters.AutoConverter".to_owned(),
            format: String::new(),
        }
    }
}

impl ExcelProperty {
    /// 创建 Java 默认参数对象。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
    /// 返回多级表头；写入时多级表头自动合并，读取时采用最后一级。
    #[must_use]
    pub fn value(&self) -> &[String] {
        &self.value
    }
    /// 设置多级表头。
    pub fn set_value(&mut self, value: impl IntoIterator<Item = impl Into<String>>) {
        self.value = value.into_iter().map(Into::into).collect();
    }
    /// 返回固定列下标，`-1` 表示未指定。
    #[must_use]
    pub const fn index(&self) -> i32 {
        self.index
    }
    /// 设置固定列下标。
    pub const fn set_index(&mut self, index: i32) {
        self.index = index;
    }
    /// 返回排序权重。
    #[must_use]
    pub const fn order(&self) -> i32 {
        self.order
    }
    /// 设置排序权重。
    pub const fn set_order(&mut self, order: i32) {
        self.order = order;
    }
    /// 返回 converter 的稳定类型名。
    #[must_use]
    pub fn converter(&self) -> &str {
        &self.converter
    }
    /// 设置 converter 的稳定类型名。
    pub fn set_converter(&mut self, converter: impl Into<String>) {
        self.converter = converter.into();
    }
    /// 返回已弃用的格式字符串。
    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }
    /// 设置已弃用的格式字符串。
    pub fn set_format(&mut self, format: impl Into<String>) {
        self.format = format.into();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_match_java() {
        let prop = ExcelProperty::new();
        assert_eq!(prop.value(), &[""]);
        assert_eq!(prop.index(), -1);
        assert_eq!(prop.order(), i32::MAX);
        assert_eq!(
            prop.converter(),
            "com.alibaba.excel.converters.AutoConverter"
        );
        assert!(prop.format().is_empty());
    }

    #[test]
    fn set_value_accepts_multiple_headers() {
        let mut prop = ExcelProperty::new();
        prop.set_value(["一级", "二级", "三级"]);
        assert_eq!(prop.value().len(), 3);
        assert_eq!(prop.value()[0], "一级");
    }

    #[test]
    fn set_index_and_order() {
        let mut prop = ExcelProperty::new();
        prop.set_index(5);
        assert_eq!(prop.index(), 5);
        prop.set_order(10);
        assert_eq!(prop.order(), 10);
    }

    #[test]
    fn set_converter_and_format() {
        let mut prop = ExcelProperty::new();
        prop.set_converter("my.Converter");
        assert_eq!(prop.converter(), "my.Converter");
        prop.set_format("yyyy-MM-dd");
        assert_eq!(prop.format(), "yyyy-MM-dd");
    }

    #[test]
    fn clone_and_eq() {
        let mut a = ExcelProperty::new();
        a.set_index(3);
        a.set_value(["Name"]);
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn debug_fmt() {
        let prop = ExcelProperty::new();
        let text = format!("{:?}", prop);
        assert!(text.contains("ExcelProperty"));
    }
}
