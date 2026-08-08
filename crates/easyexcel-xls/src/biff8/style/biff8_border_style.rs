/// BIFF8 XF 边框线型。
///
/// 对应 Java：`org.apache.poi.ss.usermodel.BorderStyle`。
#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub enum Biff8BorderStyle {
    /// 无边框。
    #[default]
    None,
    /// 细实线。
    Thin,
    /// 中等实线。
    Medium,
    /// 虚线。
    Dashed,
    /// 点线。
    Dotted,
    /// 粗实线。
    Thick,
    /// 双线。
    Double,
    /// 发丝线。
    Hair,
    /// 中等虚线。
    MediumDashed,
    /// 点划线。
    DashDot,
    /// 中等点划线。
    MediumDashDot,
    /// 双点划线。
    DashDotDot,
    /// 中等双点划线。
    MediumDashDotDot,
    /// 斜点划线。
    SlantDashDot,
}

impl Biff8BorderStyle {
    /// 返回 BIFF8 `dg` 四位编码。对应 Java：`BorderStyle#getCode()`。
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Thin => 1,
            Self::Medium => 2,
            Self::Dashed => 3,
            Self::Dotted => 4,
            Self::Thick => 5,
            Self::Double => 6,
            Self::Hair => 7,
            Self::MediumDashed => 8,
            Self::DashDot => 9,
            Self::MediumDashDot => 10,
            Self::DashDotDot => 11,
            Self::MediumDashDotDot => 12,
            Self::SlantDashDot => 13,
        }
    }
}
