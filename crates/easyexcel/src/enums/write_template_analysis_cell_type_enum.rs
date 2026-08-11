//! 对应 Java：`com.alibaba.excel.enums.WriteTemplateAnalysisCellTypeEnum`.

/// Cell kind discovered while analysing a template placeholder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.WriteTemplateAnalysisCellTypeEnum。
pub enum WriteTemplateAnalysisCellTypeEnum {
    /// Common placeholder such as `{key}`.
    Common,
    /// Collection placeholder such as `{name.field}`.
    Collection,
}

impl WriteTemplateAnalysisCellTypeEnum {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 2] = [Self::Common, Self::Collection];
    /// Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Common => "COMMON",
            Self::Collection => "COLLECTION",
        }
    }
}

impl std::str::FromStr for WriteTemplateAnalysisCellTypeEnum {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown WriteTemplateAnalysisCellTypeEnum value: {value}"))
    }
}
