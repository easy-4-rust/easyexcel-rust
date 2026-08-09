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
    /// 保留 UTF-16 字体区间语义的内联富文本。
    RichText(TemplateRichText),
    /// 显示值与由 package 层写入的超链接。
    Hyperlink {
        /// 单元格显示值。
        value: Box<TemplateCellValue>,
        /// 超链接元数据。
        hyperlink: crate::xlsx::template_fill::TemplateHyperlink,
    },
    /// 单元格显示值与由 package 层写入的图片列表。
    Images {
        /// 单元格实际显示值。
        value: Box<TemplateCellValue>,
        /// 图片及锚点元数据。
        images: Vec<crate::xlsx::template_fill::TemplateImage>,
    },
    /// 带传统 OOXML 批注的单元格；工作表 XML 渲染内部值，批注由 package 层写入。
    Comment {
        /// 单元格实际值。
        value: Box<TemplateCellValue>,
        /// 批注元数据。
        comment: crate::xlsx::template_fill::TemplateComment,
    },
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
            Self::RichText(value) => value.as_text().to_owned(),
            Self::Bool(value) => value.to_string(),
            Self::Comment { value, .. }
            | Self::Hyperlink { value, .. }
            | Self::Images { value, .. } => value.as_text(),
        }
    }
}
