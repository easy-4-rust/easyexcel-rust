//! 对应 Java：`com.alibaba.excel.enums.NumericCellTypeEnum`.
//!
//! POI-specific supplement; not surfaced publicly by Rust.

/// Supplements POI `CellType` so write paths can distinguish date from number.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.NumericCellTypeEnum。
pub enum NumericCellTypeEnum {
    /// Plain number.
    Number,
    /// Date encoded as a serial number.
    Date,
}

impl NumericCellTypeEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 2] = [Self::Number, Self::Date];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Number => "NUMBER",
            Self::Date => "DATE",
        }
    }
}

impl std::str::FromStr for NumericCellTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown NumericCellTypeEnum value: {value}"))
    }
}
