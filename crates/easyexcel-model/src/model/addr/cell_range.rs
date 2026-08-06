/// 对应 Java：无直接对应对象；Rust 架构扩展。 A rectangular range of cells, e.g. `A1:B10`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellRange {
    pub start: CellAddress,
    pub end: CellAddress,
}

impl CellRange {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn new(start: CellAddress, end: CellAddress) -> Self {
        // Normalize so start is top-left, end is bottom-right (preserving abs flags).
        let (r0, r1) = (start.row.min(end.row), start.row.max(end.row));
        let (c0, c1) = (start.col.min(end.col), start.col.max(end.col));
        CellRange {
            start: CellAddress {
                row: r0,
                col: c0,
                abs_row: start.abs_row,
                abs_col: start.abs_col,
            },
            end: CellAddress {
                row: r1,
                col: c1,
                abs_row: end.abs_row,
                abs_col: end.abs_col,
            },
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn single(addr: CellAddress) -> Self {
        CellRange {
            start: addr,
            end: addr,
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse `A1:B10` (or a single `A1`).
    #[must_use]
    pub fn parse_a1(s: &str) -> Option<CellRange> {
        if let Some((a, b)) = s.split_once(':') {
            Some(CellRange::new(
                CellAddress::parse_a1(a)?,
                CellAddress::parse_a1(b)?,
            ))
        } else {
            Some(CellRange::single(CellAddress::parse_a1(s)?))
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn to_a1(self) -> String {
        if self.start == self.end {
            self.start.to_a1()
        } else {
            format!("{}:{}", self.start.to_a1(), self.end.to_a1())
        }
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn rows(self) -> u32 {
        self.end.row - self.start.row + 1
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn cols(self) -> u32 {
        self.end.col - self.start.col + 1
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn contains(self, row: u32, col: u32) -> bool {
        row >= self.start.row && row <= self.end.row && col >= self.start.col && col <= self.end.col
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Iterate (row, col) pairs in row-major order.
    pub fn iter_cells(self) -> impl Iterator<Item = (u32, u32)> {
        let (r0, r1, c0, c1) = (self.start.row, self.end.row, self.start.col, self.end.col);
        (r0..=r1).flat_map(move |r| (c0..=c1).map(move |c| (r, c)))
    }
}

impl fmt::Display for CellRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_a1())
    }
}

