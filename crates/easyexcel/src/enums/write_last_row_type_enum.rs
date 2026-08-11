//! 对应 Java：`com.alibaba.excel.enums.WriteLastRowTypeEnum`.
//!
//! Tracks whether a worksheet has been initialized with template data or
//! remains empty.

/// State of the worksheet's last row.
///
/// Rust port of Java `WriteLastRowTypeEnum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.WriteLastRowTypeEnum。
pub enum WriteLastRowTypeEnum {
    /// Excel created without a template and nothing has been written.
    CommonEmpty,
    /// Excel created from a template and nothing has been written.
    TemplateEmpty,
    /// At least one row has been written.
    HasData,
}

impl WriteLastRowTypeEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 3] = [Self::CommonEmpty, Self::TemplateEmpty, Self::HasData];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::CommonEmpty => "COMMON_EMPTY",
            Self::TemplateEmpty => "TEMPLATE_EMPTY",
            Self::HasData => "HAS_DATA",
        }
    }
}

impl std::str::FromStr for WriteLastRowTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown WriteLastRowTypeEnum value: {value}"))
    }
}
