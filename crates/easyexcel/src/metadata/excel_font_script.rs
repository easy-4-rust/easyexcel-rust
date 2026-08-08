//! 对应 Java：`com.alibaba.excel.enums.poi.FontScript`.

/// Font script position used by annotation-driven font styles.
///
/// Java uses POI `FontScript` codes; Rust strips them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
/// 对应 Java：com.alibaba.excel.enums.poi.FontScript。
pub enum ExcelFontScript {
    /// Normal baseline text.
    None,
    /// Superscript text.
    Superscript,
    /// Subscript text.
    Subscript,
}
