//! 对应 Java：`com.alibaba.excel.enums.ReadDefaultReturnEnum`.
//!
//! `STRING` (default) / `ACTUAL_DATA` / `READ_CELL_DATA`.

/// Value mode used when reading rows without a declared Rust model.
///
/// Rust port of Java `ReadDefaultReturnEnum`. Mirrors the same three modes
/// while the `Default` impl reproduces Java's `STRING` default.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.ReadDefaultReturnEnum。
pub enum ReadDefaultReturnEnum {
    /// Convert every present cell to the text a user sees in the workbook. (Java `STRING`, default)
    #[default]
    String,
    /// Preserve the backend-neutral scalar type of each cell. (Java `ACTUAL_DATA`)
    ActualData,
    /// Return the scalar together with its raw value, location, and formula. (Java `READ_CELL_DATA`)
    ReadCellData,
}

impl ReadDefaultReturnEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 3] = [Self::String, Self::ActualData, Self::ReadCellData];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::String => "STRING",
            Self::ActualData => "ACTUAL_DATA",
            Self::ReadCellData => "READ_CELL_DATA",
        }
    }
}

impl std::str::FromStr for ReadDefaultReturnEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown ReadDefaultReturnEnum value: {value}"))
    }
}
