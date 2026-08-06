/// 对应 Java：无直接对应对象；Rust 架构扩展。 A cell or rectangular range reference.
#[derive(Debug, Clone, PartialEq)]
pub struct Reference {
    pub sheet: SheetSpec,
    pub start: CellAddress,
    /// `None` for a single cell; `Some` for an `A1:B2` range.
    pub end: Option<CellAddress>,
}

impl Reference {
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn cell(sheet: SheetSpec, addr: CellAddress) -> Reference {
        Reference {
            sheet,
            start: addr,
            end: None,
        }
    }
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn range(sheet: SheetSpec, start: CellAddress, end: CellAddress) -> Reference {
        Reference {
            sheet,
            start,
            end: Some(end),
        }
    }
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_range(&self) -> bool {
        self.end.is_some()
    }
}

