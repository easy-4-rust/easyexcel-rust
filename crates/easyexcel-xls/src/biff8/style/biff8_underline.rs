/// BIFF8 FONT 记录的下划线类型。
///
/// 对应 Java：Apache POI `FontUnderline`；EasyExcel `WriteFont#getUnderline`。
#[derive(Debug, Clone, Copy, Default, Hash, Eq, PartialEq)]
pub enum Biff8Underline {
    /// 不使用下划线。
    #[default]
    None,
    /// 单下划线。
    Single,
    /// 双下划线。
    Double,
    /// 单会计下划线。
    SingleAccounting,
    /// 双会计下划线。
    DoubleAccounting,
}

impl Biff8Underline {
    /// 返回 BIFF8 FONT `uls` 字段值。
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Single => 1,
            Self::Double => 2,
            Self::SingleAccounting => 0x21,
            Self::DoubleAccounting => 0x22,
        }
    }
}
