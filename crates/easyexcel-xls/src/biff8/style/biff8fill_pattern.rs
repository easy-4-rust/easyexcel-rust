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

#[cfg(test)]
mod biff8fill_pattern_tests {
    use super::*;

    /// 验证每个 variant 的 code() 返回正确的 BIFF8 填充图案编号。
    #[test]
    fn fill_pattern_codes_match_spec() {
        assert_eq!(Biff8FillPattern::None.code(), 0);
        assert_eq!(Biff8FillPattern::Solid.code(), 1);
        assert_eq!(Biff8FillPattern::MediumGray.code(), 2);
        assert_eq!(Biff8FillPattern::DarkGray.code(), 3);
        assert_eq!(Biff8FillPattern::LightGray.code(), 4);
        assert_eq!(Biff8FillPattern::DarkHorizontal.code(), 5);
        assert_eq!(Biff8FillPattern::DarkVertical.code(), 6);
        assert_eq!(Biff8FillPattern::DarkDown.code(), 7);
        assert_eq!(Biff8FillPattern::DarkUp.code(), 8);
        assert_eq!(Biff8FillPattern::DarkGrid.code(), 9);
        assert_eq!(Biff8FillPattern::DarkTrellis.code(), 10);
        assert_eq!(Biff8FillPattern::LightHorizontal.code(), 11);
        assert_eq!(Biff8FillPattern::LightVertical.code(), 12);
        assert_eq!(Biff8FillPattern::LightDown.code(), 13);
        assert_eq!(Biff8FillPattern::LightUp.code(), 14);
        assert_eq!(Biff8FillPattern::LightGrid.code(), 15);
        assert_eq!(Biff8FillPattern::LightTrellis.code(), 16);
        assert_eq!(Biff8FillPattern::Gray125.code(), 17);
        assert_eq!(Biff8FillPattern::Gray0625.code(), 18);
    }

    /// 验证所有 code 值在 0..=18 范围内且连续。
    #[test]
    fn fill_pattern_codes_are_contiguous() {
        let codes: Vec<u8> = [
            Biff8FillPattern::None,
            Biff8FillPattern::Solid,
            Biff8FillPattern::MediumGray,
            Biff8FillPattern::DarkGray,
            Biff8FillPattern::LightGray,
            Biff8FillPattern::DarkHorizontal,
            Biff8FillPattern::DarkVertical,
            Biff8FillPattern::DarkDown,
            Biff8FillPattern::DarkUp,
            Biff8FillPattern::DarkGrid,
            Biff8FillPattern::DarkTrellis,
            Biff8FillPattern::LightHorizontal,
            Biff8FillPattern::LightVertical,
            Biff8FillPattern::LightDown,
            Biff8FillPattern::LightUp,
            Biff8FillPattern::LightGrid,
            Biff8FillPattern::LightTrellis,
            Biff8FillPattern::Gray125,
            Biff8FillPattern::Gray0625,
        ]
        .iter()
        .map(|p| p.code())
        .collect();
        assert_eq!(codes, (0..=18).collect::<Vec<u8>>());
    }

    /// Clone, Copy, PartialEq, Eq, Debug 派生。
    #[test]
    fn traits_work() {
        let a = Biff8FillPattern::Solid;
        let b = a;
        assert_eq!(a, b);
        let _ = format!("{a:?}");
        let c = a.clone();
        assert_eq!(a, c);
    }
}

