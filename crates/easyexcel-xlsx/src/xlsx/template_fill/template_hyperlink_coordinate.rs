// 模板超链接坐标结构体。
// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。
// 从 `template_hyperlink.rs` 拆分而来，遵循"一个 .rs 文件只对应一个 Java 对象"规范。

/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。绝对或相对坐标。
///
/// 用于描述模板超链接覆盖范围中的行/列坐标，支持绝对和相对两种模式。
/// 大于零的绝对值优先；否则按相对偏移计算。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TemplateHyperlinkCoordinate {
    /// 大于零时优先使用的绝对零基坐标。
    pub absolute: Option<u32>,
    /// 相对当前填充单元格的偏移。
    pub relative: Option<i32>,
}
