/// BIFF8 语义水平对齐；数值协议码仅保留在本 crate 内部。
/// 对应 Java：`org.apache.poi.ss.usermodel.HorizontalAlignment`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8HorizontalAlignment {
    /// 按单元格值类型决定的常规对齐。
    General,
    /// 左对齐。
    Left,
    /// 居中对齐。
    Center,
    /// 右对齐。
    Right,
    /// 横向重复内容以填满单元格。
    Fill,
    /// 两端对齐。
    Justify,
    /// 跨相邻单元格居中。
    CenterAcross,
    /// 分散对齐。
    Distributed,
}

impl Biff8HorizontalAlignment {
    const fn code(self) -> u8 {
        match self {
            Self::General => 0,
            Self::Left => 1,
            Self::Center => 2,
            Self::Right => 3,
            Self::Fill => 4,
            Self::Justify => 5,
            Self::CenterAcross => 6,
            Self::Distributed => 7,
        }
    }
}

