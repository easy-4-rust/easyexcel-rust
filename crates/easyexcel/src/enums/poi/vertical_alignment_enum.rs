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
