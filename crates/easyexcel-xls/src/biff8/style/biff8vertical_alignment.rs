/// BIFF8 语义垂直对齐；数值协议码仅保留在本 crate 内部。
/// 对应 Java：`org.apache.poi.ss.usermodel.VerticalAlignment`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8VerticalAlignment {
    /// 顶部对齐。
    Top,
    /// 垂直居中。
    Center,
    /// 底部对齐。
    Bottom,
    /// 垂直两端对齐。
    Justify,
    /// 垂直分散对齐。
    Distributed,
}

impl Biff8VerticalAlignment {
    const fn code(self) -> u8 {
        match self {
            Self::Top => 0,
            Self::Center => 1,
            Self::Bottom => 2,
            Self::Justify => 3,
            Self::Distributed => 4,
        }
    }
}

