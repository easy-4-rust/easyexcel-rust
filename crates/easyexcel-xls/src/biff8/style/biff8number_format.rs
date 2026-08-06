/// 对应 Java：无直接对应对象；Rust 架构扩展。 BIFF8 数字格式描述，隔离 `EasyExcel` 注解模型与底层 XLS 引擎。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Biff8NumberFormat {
    /// Excel 内建格式索引。
    Builtin(u8),
    /// 自定义格式代码。
    Custom(String),
}

