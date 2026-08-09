//! 对应 Java：`com.alibaba.excel.enums.poi.BorderStyleEnum`。

/// Java 注解边框枚举；`Default` 保留 Java `null` 语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderStyleEnum {
    #[default]
    Default,
    None,
    Thin,
    Medium,
    Dashed,
    Dotted,
    Thick,
    Double,
    Hair,
    MediumDashed,
    DashDot,
    MediumDashDot,
    DashDotDot,
    MediumDashDotDot,
    SlantedDashDot,
}

impl BorderStyleEnum {
    /// 按 Java `values()` 声明顺序列出全部枚举值。
    pub const ALL: [Self; 15] = [
        Self::Default, Self::None, Self::Thin, Self::Medium, Self::Dashed,
        Self::Dotted, Self::Thick, Self::Double, Self::Hair, Self::MediumDashed,
        Self::DashDot, Self::MediumDashDot, Self::DashDotDot,
        Self::MediumDashDotDot, Self::SlantedDashDot,
    ];

    /// 返回 Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT", Self::None => "NONE", Self::Thin => "THIN",
            Self::Medium => "MEDIUM", Self::Dashed => "DASHED", Self::Dotted => "DOTTED",
            Self::Thick => "THICK", Self::Double => "DOUBLE", Self::Hair => "HAIR",
            Self::MediumDashed => "MEDIUM_DASHED", Self::DashDot => "DASH_DOT",
            Self::MediumDashDot => "MEDIUM_DASH_DOT", Self::DashDotDot => "DASH_DOT_DOT",
            Self::MediumDashDotDot => "MEDIUM_DASH_DOT_DOT",
            Self::SlantedDashDot => "SLANTED_DASH_DOT",
        }
    }

    /// 返回底层边框；`Default` 对应 Java `null`。
    #[must_use]
    pub const fn poi_border_style(self) -> Option<crate::ExcelBorderStyle> {
        Some(match self {
            Self::Default => return None,
            Self::None => crate::ExcelBorderStyle::None,
            Self::Thin => crate::ExcelBorderStyle::Thin,
            Self::Medium => crate::ExcelBorderStyle::Medium,
            Self::Dashed => crate::ExcelBorderStyle::Dashed,
            Self::Dotted => crate::ExcelBorderStyle::Dotted,
            Self::Thick => crate::ExcelBorderStyle::Thick,
            Self::Double => crate::ExcelBorderStyle::Double,
            Self::Hair => crate::ExcelBorderStyle::Hair,
            Self::MediumDashed => crate::ExcelBorderStyle::MediumDashed,
            Self::DashDot => crate::ExcelBorderStyle::DashDot,
            Self::MediumDashDot => crate::ExcelBorderStyle::MediumDashDot,
            Self::DashDotDot => crate::ExcelBorderStyle::DashDotDot,
            Self::MediumDashDotDot => crate::ExcelBorderStyle::MediumDashDotDot,
            Self::SlantedDashDot => crate::ExcelBorderStyle::SlantDashDot,
        })
    }

    /// Java Lombok `getPoiBorderStyle` 兼容别名。
    #[must_use]
    pub const fn get_poi_border_style(self) -> Option<crate::ExcelBorderStyle> {
        self.poi_border_style()
    }
}

impl std::str::FromStr for BorderStyleEnum {
    type Err = String;

    /// 解析 Java `valueOf(String)` 使用的精确枚举常量名。
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown BorderStyleEnum value: {value}"))
    }
}
