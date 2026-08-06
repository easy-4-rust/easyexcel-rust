/// BIFF8 语义填充图案。
/// 对应 Java：`org.apache.poi.ss.usermodel.FillPatternType`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8FillPattern {
    /// 无填充。
    None,
    /// 纯色填充。
    Solid,
    /// 中灰填充。
    MediumGray,
    /// 深灰填充。
    DarkGray,
    /// 浅灰填充。
    LightGray,
    /// 深色水平线。
    DarkHorizontal,
    /// 深色垂直线。
    DarkVertical,
    /// 深色向下斜线。
    DarkDown,
    /// 深色向上斜线。
    DarkUp,
    /// 深色网格。
    DarkGrid,
    /// 深色格架。
    DarkTrellis,
    /// 浅色水平线。
    LightHorizontal,
    /// 浅色垂直线。
    LightVertical,
    /// 浅色向下斜线。
    LightDown,
    /// 浅色向上斜线。
    LightUp,
    /// 浅色网格。
    LightGrid,
    /// 浅色格架。
    LightTrellis,
    /// 12.5% 灰色填充。
    Gray125,
    /// 6.25% 灰色填充。
    Gray0625,
}

impl Biff8FillPattern {
    const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Solid => 1,
            Self::MediumGray => 2,
            Self::DarkGray => 3,
            Self::LightGray => 4,
            Self::DarkHorizontal => 5,
            Self::DarkVertical => 6,
            Self::DarkDown => 7,
            Self::DarkUp => 8,
            Self::DarkGrid => 9,
            Self::DarkTrellis => 10,
            Self::LightHorizontal => 11,
            Self::LightVertical => 12,
            Self::LightDown => 13,
            Self::LightUp => 14,
            Self::LightGrid => 15,
            Self::LightTrellis => 16,
            Self::Gray125 => 17,
            Self::Gray0625 => 18,
        }
    }
}

