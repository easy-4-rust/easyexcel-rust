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
    pub fn new() -> Self { Self::default() }
    /// 返回多级表头；写入时多级表头自动合并，读取时采用最后一级。
    #[must_use]
    pub fn value(&self) -> &[String] { &self.value }
    /// 设置多级表头。
    pub fn set_value(&mut self, value: impl IntoIterator<Item = impl Into<String>>) {
        self.value = value.into_iter().map(Into::into).collect();
    }
    /// 返回固定列下标，`-1` 表示未指定。
    #[must_use]
    pub const fn index(&self) -> i32 { self.index }
    /// 设置固定列下标。
    pub const fn set_index(&mut self, index: i32) { self.index = index; }
    /// 返回排序权重。
    #[must_use]
    pub const fn order(&self) -> i32 { self.order }
    /// 设置排序权重。
    pub const fn set_order(&mut self, order: i32) { self.order = order; }
    /// 返回 converter 的稳定类型名。
    #[must_use]
    pub fn converter(&self) -> &str { &self.converter }
    /// 设置 converter 的稳定类型名。
    pub fn set_converter(&mut self, converter: impl Into<String>) { self.converter = converter.into(); }
    /// 返回已弃用的格式字符串。
    #[must_use]
    pub fn format(&self) -> &str { &self.format }
    /// 设置已弃用的格式字符串。
    pub fn set_format(&mut self, format: impl Into<String>) { self.format = format.into(); }
}
