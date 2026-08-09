//! 对应 Java：`com.alibaba.excel.enums.WriteDirectionEnum`.
//!
//! `VERTICAL` vs `HORIZONTAL` for template fills.
//!
//! Java uses this enum; the `easyexcel-template` crate uses
//! `easyexcel_template::FillDirection` which already provides the same two
//! variants. This enum is kept as a type alias to avoid diverging names.

/// Direction in which a template fill expands.
///
/// Rust port of Java `WriteDirectionEnum`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.WriteDirectionEnum。
pub enum WriteDirectionEnum {
    /// Expand downward.
    Vertical,
    /// Expand rightward.
    Horizontal,
}

impl WriteDirectionEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 2] = [Self::Vertical, Self::Horizontal];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self { Self::Vertical => "VERTICAL", Self::Horizontal => "HORIZONTAL" }
    }
}

impl std::str::FromStr for WriteDirectionEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown WriteDirectionEnum value: {value}"))
    }
}
