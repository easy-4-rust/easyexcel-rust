//! 对应 Java：`com.alibaba.excel.enums.poi.FillPatternTypeEnum`。

/// Java 注解填充图案枚举；`Default` 保留 Java `null` 语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FillPatternTypeEnum {
    #[default]
    Default,
    NoFill,
    SolidForeground,
    FineDots,
    AltBars,
    SparseDots,
    ThickHorzBands,
    ThickVertBands,
    ThickBackwardDiag,
    ThickForwardDiag,
    BigSpots,
    Bricks,
    ThinHorzBands,
    ThinVertBands,
    ThinBackwardDiag,
    ThinForwardDiag,
    Squares,
    Diamonds,
    LessDots,
    LeastDots,
}

impl FillPatternTypeEnum {
    /// 返回底层填充图案；`Default` 对应 Java `null`。
    #[must_use]
    pub const fn poi_fill_pattern_type(self) -> Option<crate::ExcelFillPattern> {
        Some(match self {
            Self::Default => return None,
            Self::NoFill => crate::ExcelFillPattern::None,
            Self::SolidForeground => crate::ExcelFillPattern::Solid,
            Self::FineDots => crate::ExcelFillPattern::MediumGray,
            Self::AltBars => crate::ExcelFillPattern::DarkGray,
            Self::SparseDots => crate::ExcelFillPattern::LightGray,
            Self::ThickHorzBands => crate::ExcelFillPattern::DarkHorizontal,
            Self::ThickVertBands => crate::ExcelFillPattern::DarkVertical,
            Self::ThickBackwardDiag => crate::ExcelFillPattern::DarkDown,
            Self::ThickForwardDiag => crate::ExcelFillPattern::DarkUp,
            Self::BigSpots => crate::ExcelFillPattern::DarkGrid,
            Self::Bricks => crate::ExcelFillPattern::DarkTrellis,
            Self::ThinHorzBands => crate::ExcelFillPattern::LightHorizontal,
            Self::ThinVertBands => crate::ExcelFillPattern::LightVertical,
            Self::ThinBackwardDiag => crate::ExcelFillPattern::LightDown,
            Self::ThinForwardDiag => crate::ExcelFillPattern::LightUp,
            Self::Squares => crate::ExcelFillPattern::LightGrid,
            Self::Diamonds => crate::ExcelFillPattern::LightTrellis,
            Self::LessDots => crate::ExcelFillPattern::Gray125,
            Self::LeastDots => crate::ExcelFillPattern::Gray0625,
        })
    }

    /// Java Lombok `getPoiFillPatternType` 兼容别名。
    #[must_use]
    pub const fn get_poi_fill_pattern_type(self) -> Option<crate::ExcelFillPattern> {
        self.poi_fill_pattern_type()
    }
}
