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
    /// 按 Java `values()` 声明顺序列出全部枚举值。
    pub const ALL: [Self; 20] = [
        Self::Default, Self::NoFill, Self::SolidForeground, Self::FineDots,
        Self::AltBars, Self::SparseDots, Self::ThickHorzBands, Self::ThickVertBands,
        Self::ThickBackwardDiag, Self::ThickForwardDiag, Self::BigSpots, Self::Bricks,
        Self::ThinHorzBands, Self::ThinVertBands, Self::ThinBackwardDiag,
        Self::ThinForwardDiag, Self::Squares, Self::Diamonds, Self::LessDots,
        Self::LeastDots,
    ];

    /// 返回 Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT", Self::NoFill => "NO_FILL",
            Self::SolidForeground => "SOLID_FOREGROUND", Self::FineDots => "FINE_DOTS",
            Self::AltBars => "ALT_BARS", Self::SparseDots => "SPARSE_DOTS",
            Self::ThickHorzBands => "THICK_HORZ_BANDS", Self::ThickVertBands => "THICK_VERT_BANDS",
            Self::ThickBackwardDiag => "THICK_BACKWARD_DIAG",
            Self::ThickForwardDiag => "THICK_FORWARD_DIAG", Self::BigSpots => "BIG_SPOTS",
            Self::Bricks => "BRICKS", Self::ThinHorzBands => "THIN_HORZ_BANDS",
            Self::ThinVertBands => "THIN_VERT_BANDS", Self::ThinBackwardDiag => "THIN_BACKWARD_DIAG",
            Self::ThinForwardDiag => "THIN_FORWARD_DIAG", Self::Squares => "SQUARES",
            Self::Diamonds => "DIAMONDS", Self::LessDots => "LESS_DOTS", Self::LeastDots => "LEAST_DOTS",
        }
    }

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

impl std::str::FromStr for FillPatternTypeEnum {
    type Err = String;

    /// 解析 Java `valueOf(String)` 使用的精确枚举常量名。
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown FillPatternTypeEnum value: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_twenty_variants() {
        assert_eq!(FillPatternTypeEnum::ALL.len(), 20);
    }

    #[test]
    fn default_is_default_variant() {
        assert_eq!(FillPatternTypeEnum::default(), FillPatternTypeEnum::Default);
    }

    #[test]
    fn java_name_covers_all_variants() {
        let expected = [
            "DEFAULT", "NO_FILL", "SOLID_FOREGROUND", "FINE_DOTS",
            "ALT_BARS", "SPARSE_DOTS", "THICK_HORZ_BANDS", "THICK_VERT_BANDS",
            "THICK_BACKWARD_DIAG", "THICK_FORWARD_DIAG", "BIG_SPOTS", "BRICKS",
            "THIN_HORZ_BANDS", "THIN_VERT_BANDS", "THIN_BACKWARD_DIAG",
            "THIN_FORWARD_DIAG", "SQUARES", "DIAMONDS", "LESS_DOTS", "LEAST_DOTS",
        ];
        for (variant, name) in FillPatternTypeEnum::ALL.iter().zip(expected.iter()) {
            assert_eq!(variant.java_name(), *name);
        }
    }

    #[test]
    fn default_returns_none_for_poi_fill_pattern() {
        assert_eq!(FillPatternTypeEnum::Default.poi_fill_pattern_type(), None);
    }

    #[test]
    fn non_default_returns_some() {
        assert!(FillPatternTypeEnum::SolidForeground.poi_fill_pattern_type().is_some());
        assert!(FillPatternTypeEnum::FineDots.poi_fill_pattern_type().is_some());
        assert!(FillPatternTypeEnum::Squares.poi_fill_pattern_type().is_some());
    }

    #[test]
    fn get_poi_fill_pattern_type_matches_poi_fill_pattern_type() {
        for variant in FillPatternTypeEnum::ALL {
            assert_eq!(variant.get_poi_fill_pattern_type(), variant.poi_fill_pattern_type());
        }
    }

    #[test]
    fn solid_foreground_maps_to_solid() {
        assert_eq!(
            FillPatternTypeEnum::SolidForeground.poi_fill_pattern_type(),
            Some(crate::ExcelFillPattern::Solid)
        );
    }

    #[test]
    fn no_fill_maps_to_none() {
        assert_eq!(
            FillPatternTypeEnum::NoFill.poi_fill_pattern_type(),
            Some(crate::ExcelFillPattern::None)
        );
    }

    #[test]
    fn squares_maps_to_light_grid() {
        assert_eq!(
            FillPatternTypeEnum::Squares.poi_fill_pattern_type(),
            Some(crate::ExcelFillPattern::LightGrid)
        );
    }

    #[test]
    fn diamonds_maps_to_light_trellis() {
        assert_eq!(
            FillPatternTypeEnum::Diamonds.poi_fill_pattern_type(),
            Some(crate::ExcelFillPattern::LightTrellis)
        );
    }

    #[test]
    fn least_dots_maps_to_gray0625() {
        assert_eq!(
            FillPatternTypeEnum::LeastDots.poi_fill_pattern_type(),
            Some(crate::ExcelFillPattern::Gray0625)
        );
    }

    #[test]
    fn from_str_parses_all_variants() {
        for variant in FillPatternTypeEnum::ALL {
            let name = variant.java_name();
            let parsed: FillPatternTypeEnum = name.parse().unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!("INVALID".parse::<FillPatternTypeEnum>().is_err());
    }

    #[test]
    fn from_str_error_contains_input() {
        let err = "BOGUS".parse::<FillPatternTypeEnum>().unwrap_err();
        assert!(err.contains("BOGUS"), "error should contain input: {err}");
    }

    #[test]
    fn clone_copy_eq() {
        let a = FillPatternTypeEnum::Bricks;
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_contains_variant_name() {
        let text = format!("{:?}", FillPatternTypeEnum::ThickBackwardDiag);
        assert!(text.contains("ThickBackwardDiag"));
    }
}
