//! 对应 Java：`com.alibaba.excel.enums.RowTypeEnum`.
//!
//! Used to distinguish data rows from empty rows during SAX streaming.

/// The types of row.
///
/// Rust port of Java `RowTypeEnum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.RowTypeEnum。
pub enum RowTypeEnum {
    /// Data row.                  (Java `DATA`)
    Data,
    /// Empty row (only empty cells). (Java `EMPTY`)
    Empty,
}

impl RowTypeEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 2] = [Self::Data, Self::Empty];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Data => "DATA",
            Self::Empty => "EMPTY",
        }
    }
}

impl std::str::FromStr for RowTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown RowTypeEnum value: {value}"))
    }
}
