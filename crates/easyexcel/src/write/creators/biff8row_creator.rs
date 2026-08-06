/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct Biff8RowCreator<'a> {
    pub(crate) sheet: &'a mut Biff8Sheet,
}

impl RowCreator for Biff8RowCreator<'_> {
    type Row<'a>
        = Biff8Row<'a>
    where
        Self: 'a;

    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>> {
        Biff8Sheet::validate_row_index(row_index).map_err(ExcelError::from)?;
        Ok(Biff8Row {
            sheet: self.sheet,
            row_index,
        })
    }
}

