//! 工作簿无关的 Excel 数据格式选择。

/// 对应 Java：无直接对应对象；Rust 架构扩展。 内建数字格式索引或自定义 Excel 格式文本。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExcelDataFormat {
    /// Apache POI / `EasyExcel` 内建格式索引。
    Builtin(u8),
    /// 自定义 Excel 数字格式文本。
    Custom(&'static str),
}
