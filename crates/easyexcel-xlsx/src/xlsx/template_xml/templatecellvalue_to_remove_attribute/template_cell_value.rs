/// 对应 Java：无直接对应对象；Rust 架构扩展。 可直接写入 `SpreadsheetML` 单元格的中立值。
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateCellValue {
    /// 空单元格。
    Empty,
    /// 内联字符串。
    Text(String),
    /// 布尔值。
    Bool(bool),
    /// 已验证的数字词法值。
    Number(String),
    /// ISO 8601 日期或日期时间。
    Date(String),
    /// 不含外层 `<f>` 的公式表达式。
    Formula(String),
    /// Excel 错误文本。
    Error(String),
}

impl TemplateCellValue {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回适合占位符字符串替换的显示文本。
    #[must_use]
    pub fn as_text(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Text(value)
            | Self::Number(value)
            | Self::Date(value)
            | Self::Formula(value)
            | Self::Error(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
        }
    }
}

