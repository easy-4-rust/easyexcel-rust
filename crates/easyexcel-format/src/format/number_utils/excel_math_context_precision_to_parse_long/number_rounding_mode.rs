/// 对应 Java：无直接对应对象；Rust 架构扩展。 Java `java.math.RoundingMode` 对应的中立舍入模式。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum NumberRoundingMode {
    /// 远离零。
    Up,
    /// 趋近零。
    Down,
    /// 趋向正无穷。
    Ceiling,
    /// 趋向负无穷。
    Floor,
    /// 最近值，中点远离零。
    #[default]
    HalfUp,
    /// 最近值，中点趋近零。
    HalfDown,
    /// 最近值，中点取偶数邻居。
    HalfEven,
    /// 需要舍入时返回错误。
    Unnecessary,
}

impl NumberRoundingMode {
    /// 返回 `bigdecimal` 舍入模式；`Unnecessary` 由调用方显式校验。
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub const fn bigdecimal(self) -> Option<RoundingMode> {
        match self {
            Self::Up => Some(RoundingMode::Up),
            Self::Down => Some(RoundingMode::Down),
            Self::Ceiling => Some(RoundingMode::Ceiling),
            Self::Floor => Some(RoundingMode::Floor),
            Self::HalfUp => Some(RoundingMode::HalfUp),
            Self::HalfDown => Some(RoundingMode::HalfDown),
            Self::HalfEven => Some(RoundingMode::HalfEven),
            Self::Unnecessary => None,
        }
    }
}

impl From<RoundingMode> for NumberRoundingMode {
    fn from(value: RoundingMode) -> Self {
        match value {
            RoundingMode::Up => Self::Up,
            RoundingMode::Down => Self::Down,
            RoundingMode::Ceiling => Self::Ceiling,
            RoundingMode::Floor => Self::Floor,
            RoundingMode::HalfUp => Self::HalfUp,
            RoundingMode::HalfDown => Self::HalfDown,
            RoundingMode::HalfEven => Self::HalfEven,
        }
    }
}
