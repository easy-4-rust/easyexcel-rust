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

impl CellExtraTypeEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 3] = [Self::Comment, Self::Hyperlink, Self::Merge];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self { Self::Comment => "COMMENT", Self::Hyperlink => "HYPERLINK", Self::Merge => "MERGE" }
    }
}

impl std::str::FromStr for CellExtraTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown CellExtraTypeEnum value: {value}"))
    }
}
