//! 对应 Java：`com.alibaba.excel.enums.WriteTypeEnum`.
//!
//! `ADD` vs `FILL`. Used internally by `ExcelBuilderImpl` (Java) to switch
//! between `ExcelWriteAddExecutor` and `ExcelWriteFillExecutor`.

/// Write mode flag.
///
/// Rust port of Java `WriteTypeEnum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.WriteTypeEnum。
pub enum WriteTypeEnum {
    /// Append new rows. (Java `ADD`)
    Add,
    /// Fill template placeholders. (Java `FILL`)
    Fill,
}

impl WriteTypeEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 2] = [Self::Add, Self::Fill];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self { Self::Add => "ADD", Self::Fill => "FILL" }
    }
}

impl std::str::FromStr for WriteTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown WriteTypeEnum value: {value}"))
    }
}
