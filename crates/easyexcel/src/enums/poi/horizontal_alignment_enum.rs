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
