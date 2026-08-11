//! 对应 Java：`com.alibaba.excel.enums.poi.VerticalAlignmentEnum`。

/// Java 注解垂直对齐枚举；`Default` 保留 Java `null` 语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VerticalAlignmentEnum {
    #[default]
    Default,
    Top,
    Center,
    Bottom,
    Justify,
    Distributed,
}

impl VerticalAlignmentEnum {
    /// 按 Java `values()` 声明顺序列出全部枚举值。
    pub const ALL: [Self; 6] = [
        Self::Default,
        Self::Top,
        Self::Center,
        Self::Bottom,
        Self::Justify,
        Self::Distributed,
    ];

    /// 返回 Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT",
            Self::Top => "TOP",
            Self::Center => "CENTER",
            Self::Bottom => "BOTTOM",
            Self::Justify => "JUSTIFY",
            Self::Distributed => "DISTRIBUTED",
        }
    }

    /// 返回底层垂直对齐；`Default` 对应 Java `null`。
    #[must_use]
    pub const fn poi_vertical_alignment_enum(self) -> Option<crate::ExcelVerticalAlignment> {
        Some(match self {
            Self::Default => return None,
            Self::Top => crate::ExcelVerticalAlignment::Top,
            Self::Center => crate::ExcelVerticalAlignment::Center,
            Self::Bottom => crate::ExcelVerticalAlignment::Bottom,
            Self::Justify => crate::ExcelVerticalAlignment::Justify,
            Self::Distributed => crate::ExcelVerticalAlignment::Distributed,
        })
    }

    /// Java Lombok getter 兼容别名。
    #[must_use]
    pub const fn get_poi_vertical_alignment_enum(self) -> Option<crate::ExcelVerticalAlignment> {
        self.poi_vertical_alignment_enum()
    }
}

impl std::str::FromStr for VerticalAlignmentEnum {
    type Err = String;

    /// 解析 Java `valueOf(String)` 使用的精确枚举常量名。
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown VerticalAlignmentEnum value: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_name_all_variants() {
        assert_eq!(VerticalAlignmentEnum::Default.java_name(), "DEFAULT");
        assert_eq!(VerticalAlignmentEnum::Top.java_name(), "TOP");
        assert_eq!(VerticalAlignmentEnum::Center.java_name(), "CENTER");
        assert_eq!(VerticalAlignmentEnum::Bottom.java_name(), "BOTTOM");
        assert_eq!(VerticalAlignmentEnum::Justify.java_name(), "JUSTIFY");
        assert_eq!(
            VerticalAlignmentEnum::Distributed.java_name(),
            "DISTRIBUTED"
        );
    }

    #[test]
    fn poi_vertical_alignment_default_is_none() {
        assert!(
            VerticalAlignmentEnum::Default
                .poi_vertical_alignment_enum()
                .is_none()
        );
    }

    #[test]
    fn poi_vertical_alignment_non_default() {
        assert!(
            VerticalAlignmentEnum::Top
                .poi_vertical_alignment_enum()
                .is_some()
        );
        assert!(
            VerticalAlignmentEnum::Bottom
                .poi_vertical_alignment_enum()
                .is_some()
        );
    }

    #[test]
    fn from_str_valid() {
        assert_eq!(
            "TOP".parse::<VerticalAlignmentEnum>(),
            Ok(VerticalAlignmentEnum::Top)
        );
        assert_eq!(
            "BOTTOM".parse::<VerticalAlignmentEnum>(),
            Ok(VerticalAlignmentEnum::Bottom)
        );
    }

    #[test]
    fn from_str_invalid() {
        assert!("INVALID".parse::<VerticalAlignmentEnum>().is_err());
    }

    #[test]
    fn all_has_6_variants() {
        assert_eq!(VerticalAlignmentEnum::ALL.len(), 6);
    }
}
