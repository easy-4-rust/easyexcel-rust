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
        Self::Default,
        Self::None,
        Self::Thin,
        Self::Medium,
        Self::Dashed,
        Self::Dotted,
        Self::Thick,
        Self::Double,
        Self::Hair,
        Self::MediumDashed,
        Self::DashDot,
        Self::MediumDashDot,
        Self::DashDotDot,
        Self::MediumDashDotDot,
        Self::SlantedDashDot,
    ];

    /// 返回 Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::None => "NONE",
            Self::Thin => "THIN",
            Self::Medium => "MEDIUM",
            Self::Dashed => "DASHED",
            Self::Dotted => "DOTTED",
            Self::Thick => "THICK",
            Self::Double => "DOUBLE",
            Self::Hair => "HAIR",
            Self::MediumDashed => "MEDIUM_DASHED",
            Self::DashDot => "DASH_DOT",
            Self::MediumDashDot => "MEDIUM_DASH_DOT",
            Self::DashDotDot => "DASH_DOT_DOT",
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
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown BorderStyleEnum value: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_fifteen_variants() {
        assert_eq!(BorderStyleEnum::ALL.len(), 15);
    }

    #[test]
    fn default_is_default_variant() {
        assert_eq!(BorderStyleEnum::default(), BorderStyleEnum::Default);
    }

    #[test]
    fn java_name_covers_all_variants() {
        let expected = [
            "DEFAULT",
            "NONE",
            "THIN",
            "MEDIUM",
            "DASHED",
            "DOTTED",
            "THICK",
            "DOUBLE",
            "HAIR",
            "MEDIUM_DASHED",
            "DASH_DOT",
            "MEDIUM_DASH_DOT",
            "DASH_DOT_DOT",
            "MEDIUM_DASH_DOT_DOT",
            "SLANTED_DASH_DOT",
        ];
        for (variant, name) in BorderStyleEnum::ALL.iter().zip(expected.iter()) {
            assert_eq!(variant.java_name(), *name);
        }
    }

    #[test]
    fn default_returns_none_for_poi_border_style() {
        assert_eq!(BorderStyleEnum::Default.poi_border_style(), None);
    }

    #[test]
    fn non_default_returns_some() {
        assert!(BorderStyleEnum::Thin.poi_border_style().is_some());
        assert!(BorderStyleEnum::Double.poi_border_style().is_some());
        assert!(BorderStyleEnum::SlantedDashDot.poi_border_style().is_some());
    }

    #[test]
    fn get_poi_border_style_matches_poi_border_style() {
        for variant in BorderStyleEnum::ALL {
            assert_eq!(variant.get_poi_border_style(), variant.poi_border_style());
        }
    }

    #[test]
    fn thin_maps_to_thin() {
        assert_eq!(
            BorderStyleEnum::Thin.poi_border_style(),
            Some(crate::ExcelBorderStyle::Thin)
        );
    }

    #[test]
    fn medium_maps_to_medium() {
        assert_eq!(
            BorderStyleEnum::Medium.poi_border_style(),
            Some(crate::ExcelBorderStyle::Medium)
        );
    }

    #[test]
    fn double_maps_to_double() {
        assert_eq!(
            BorderStyleEnum::Double.poi_border_style(),
            Some(crate::ExcelBorderStyle::Double)
        );
    }

    #[test]
    fn none_maps_to_none() {
        assert_eq!(
            BorderStyleEnum::None.poi_border_style(),
            Some(crate::ExcelBorderStyle::None)
        );
    }

    #[test]
    fn slanted_dash_dot_maps_to_slant_dash_dot() {
        assert_eq!(
            BorderStyleEnum::SlantedDashDot.poi_border_style(),
            Some(crate::ExcelBorderStyle::SlantDashDot)
        );
    }

    #[test]
    fn from_str_parses_all_variants() {
        for variant in BorderStyleEnum::ALL {
            let name = variant.java_name();
            let parsed: BorderStyleEnum = name.parse().unwrap();
            assert_eq!(parsed, variant);
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!("UNKNOWN".parse::<BorderStyleEnum>().is_err());
    }

    #[test]
    fn from_str_error_contains_input() {
        let err = "BOGUS".parse::<BorderStyleEnum>().unwrap_err();
        assert!(err.contains("BOGUS"), "error should contain input: {err}");
    }

    #[test]
    fn clone_copy_eq() {
        let a = BorderStyleEnum::DashDot;
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn debug_contains_variant_name() {
        let text = format!("{:?}", BorderStyleEnum::MediumDashDotDot);
        assert!(text.contains("MediumDashDotDot"));
    }
}
