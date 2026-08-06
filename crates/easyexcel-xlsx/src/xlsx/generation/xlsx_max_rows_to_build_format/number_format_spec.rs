/// 对应 Java：无直接对应对象；Rust 架构扩展。 XLSX 数字格式描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumberFormatSpec {
    /// Excel 内置数字格式编号。
    Builtin(u8),
    /// 自定义数字格式代码。
    Custom(String),
}

