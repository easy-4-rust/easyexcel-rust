//! 对应 Java：`com.alibaba.excel.enums.HeadKindEnum`.

/// The types of header.
///
/// Rust port of Java `HeadKindEnum`. Distinguishes no-header, class-driven
/// headers, and ad-hoc string-list headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.HeadKindEnum。
pub enum HeadKindEnum {
    /// No header configured.
    None,
    /// Header derived from a `#[derive(ExcelRow)]` class.
    Class,
    /// Header derived from a literal string list.
    String,
}

impl HeadKindEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 3] = [Self::None, Self::Class, Self::String];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self { Self::None => "NONE", Self::Class => "CLASS", Self::String => "STRING" }
    }
}

impl std::str::FromStr for HeadKindEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown HeadKindEnum value: {value}"))
    }
}
