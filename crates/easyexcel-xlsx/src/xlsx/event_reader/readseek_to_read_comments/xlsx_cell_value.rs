/// 对应 Java：无直接对应对象；Rust 架构扩展。 中立的 XLSX 单元格缓存值。
#[derive(Debug, Clone, PartialEq)]
pub enum XlsxCellValue {
    /// 空单元格。
    Empty,
    /// 字符串。
    String(String),
    /// 布尔值。
    Bool(bool),
    /// Excel 错误文本。
    Error(String),
    /// 数字值。
    Number(f64),
}

