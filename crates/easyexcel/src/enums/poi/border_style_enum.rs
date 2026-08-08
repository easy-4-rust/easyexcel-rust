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
