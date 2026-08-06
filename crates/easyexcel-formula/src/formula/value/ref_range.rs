/// 对应 Java：无直接对应对象；Rust 架构扩展。 A concrete, resolved range reference (sheet index already looked up).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefRange {
    pub sheet: usize,
    pub start_row: u32,
    pub start_col: u32,
    pub end_row: u32,
    pub end_col: u32,
}

impl RefRange {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn single(sheet: usize, row: u32, col: u32) -> Self {
        RefRange {
            sheet,
            start_row: row,
            start_col: col,
            end_row: row,
            end_col: col,
        }
    }
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn rows(&self) -> u32 {
        self.end_row - self.start_row + 1
    }
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn cols(&self) -> u32 {
        self.end_col - self.start_col + 1
    }
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_single(&self) -> bool {
        self.start_row == self.end_row && self.start_col == self.end_col
    }
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        let (r0, r1, c0, c1) = (self.start_row, self.end_row, self.start_col, self.end_col);
        (r0..=r1).flat_map(move |r| (c0..=c1).map(move |c| (r, c)))
    }
}

