//! 对应 Java：`com.alibaba.excel.enums.poi.HorizontalAlignmentEnum`。

/// Java 注解水平对齐枚举；`Default` 保留 Java `null` 语义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HorizontalAlignmentEnum {
    #[default]
    Default,
    General,
    Left,
    Center,
    Right,
    Fill,
    Justify,
    CenterSelection,
    Distributed,
}

impl HorizontalAlignmentEnum {
    /// 按 Java `values()` 声明顺序列出全部枚举值。
    pub const ALL: [Self; 9] = [
        Self::Default, Self::General, Self::Left, Self::Center, Self::Right,
        Self::Fill, Self::Justify, Self::CenterSelection, Self::Distributed,
    ];

    /// 返回 Java 枚举常量名。
    #[must_use]
    pub const fn java_name(self) -> &'static str {
        match self {
            Self::Default => "DEFAULT", Self::General => "GENERAL", Self::Left => "LEFT",
            Self::Center => "CENTER", Self::Right => "RIGHT", Self::Fill => "FILL",
            Self::Justify => "JUSTIFY", Self::CenterSelection => "CENTER_SELECTION",
            Self::Distributed => "DISTRIBUTED",
        }
    }

    /// 返回底层水平对齐；`Default` 对应 Java `null`。
    #[must_use]
    pub const fn poi_horizontal_alignment(self) -> Option<crate::ExcelHorizontalAlignment> {
        Some(match self {
            Self::Default => return None,
            Self::General => crate::ExcelHorizontalAlignment::General,
            Self::Left => crate::ExcelHorizontalAlignment::Left,
            Self::Center => crate::ExcelHorizontalAlignment::Center,
            Self::Right => crate::ExcelHorizontalAlignment::Right,
            Self::Fill => crate::ExcelHorizontalAlignment::Fill,
            Self::Justify => crate::ExcelHorizontalAlignment::Justify,
            Self::CenterSelection => crate::ExcelHorizontalAlignment::CenterAcross,
            Self::Distributed => crate::ExcelHorizontalAlignment::Distributed,
        })
    }

    /// Java Lombok `getPoiHorizontalAlignment` 兼容别名。
    #[must_use]
    pub const fn get_poi_horizontal_alignment(self) -> Option<crate::ExcelHorizontalAlignment> {
        self.poi_horizontal_alignment()
    }
}

impl std::str::FromStr for HorizontalAlignmentEnum {
    type Err = String;

    /// 解析 Java `valueOf(String)` 使用的精确枚举常量名。
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown HorizontalAlignmentEnum value: {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_name_all_variants() {
        assert_eq!(HorizontalAlignmentEnum::Default.java_name(), "DEFAULT");
        assert_eq!(HorizontalAlignmentEnum::General.java_name(), "GENERAL");
        assert_eq!(HorizontalAlignmentEnum::Left.java_name(), "LEFT");
        assert_eq!(HorizontalAlignmentEnum::Center.java_name(), "CENTER");
        assert_eq!(HorizontalAlignmentEnum::Right.java_name(), "RIGHT");
        assert_eq!(HorizontalAlignmentEnum::Fill.java_name(), "FILL");
        assert_eq!(HorizontalAlignmentEnum::Justify.java_name(), "JUSTIFY");
        assert_eq!(HorizontalAlignmentEnum::CenterSelection.java_name(), "CENTER_SELECTION");
        assert_eq!(HorizontalAlignmentEnum::Distributed.java_name(), "DISTRIBUTED");
    }

    #[test]
    fn poi_horizontal_alignment_default_is_none() {
        assert!(HorizontalAlignmentEnum::Default.poi_horizontal_alignment().is_none());
    }

    #[test]
    fn poi_horizontal_alignment_non_default() {
        assert!(HorizontalAlignmentEnum::Left.poi_horizontal_alignment().is_some());
        assert!(HorizontalAlignmentEnum::Center.poi_horizontal_alignment().is_some());
        assert!(HorizontalAlignmentEnum::Right.poi_horizontal_alignment().is_some());
    }

    #[test]
    fn get_poi_horizontal_alignment_alias() {
        assert_eq!(
            HorizontalAlignmentEnum::Left.get_poi_horizontal_alignment(),
            HorizontalAlignmentEnum::Left.poi_horizontal_alignment()
        );
    }

    #[test]
    fn from_str_valid() {
        assert_eq!("LEFT".parse::<HorizontalAlignmentEnum>(), Ok(HorizontalAlignmentEnum::Left));
        assert_eq!("CENTER".parse::<HorizontalAlignmentEnum>(), Ok(HorizontalAlignmentEnum::Center));
    }

    #[test]
    fn from_str_invalid() {
        assert!("INVALID".parse::<HorizontalAlignmentEnum>().is_err());
    }

    #[test]
    fn all_has_9_variants() {
        assert_eq!(HorizontalAlignmentEnum::ALL.len(), 9);
    }

    #[test]
    fn default_is_default() {
        assert_eq!(HorizontalAlignmentEnum::default(), HorizontalAlignmentEnum::Default);
    }
}
