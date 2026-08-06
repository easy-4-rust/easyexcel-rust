/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct Biff8Row<'a> {
    pub(crate) sheet: &'a mut Biff8Sheet,
    pub(crate) row_index: u32,
}

impl CellCreator for Biff8Row<'_> {
    type Cell<'a>
        = Biff8CellHandle<'a>
    where
        Self: 'a;

    fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>> {
        Biff8Sheet::column_index(usize::from(column_index)).map_err(ExcelError::from)?;
        Ok(Biff8CellHandle {
            sheet: self.sheet,
            row_index: self.row_index,
            column_index,
        })
    }
}

