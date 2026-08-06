//! 对应 Java：`com.alibaba.excel.enums.CellExtraTypeEnum`.
//!
//! `COMMENT / HYPERLINK / MERGE`.

/// Extra worksheet information selectable during a read.
///
/// Rust port of Java `CellExtraTypeEnum`. Variant names are normalised to
/// `PascalCase` to match `CellExtra` callers while preserving semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 对应 Java：com.alibaba.excel.enums.CellExtraTypeEnum。
pub enum CellExtraTypeEnum {
    /// A cell comment/note.                  (Java `COMMENT`)
    Comment,
    /// A cell or range hyperlink.             (Java `HYPERLINK`)
    Hyperlink,
    /// A merged-cell range.                  (Java `MERGE`)
    Merge,
}
