/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct Biff8CellHandle<'a> {
    pub(crate) sheet: &'a mut Biff8Sheet,
    pub(crate) row_index: u32,
    pub(crate) column_index: u16,
}

impl Biff8CellHandle<'_> {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn set(self, cell: Biff8Cell) -> Result<()> {
        self.sheet
            .set(self.row_index, usize::from(self.column_index), cell)?;
        Ok(())
    }
}

