/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct XlsxRowCreator<'a> {
    pub(crate) worksheet: &'a mut Worksheet,
}

impl RowCreator for XlsxRowCreator<'_> {
    type Row<'a>
        = XlsxRow<'a>
    where
        Self: 'a;

    fn create_row(&mut self, row_index: u32) -> Result<Self::Row<'_>> {
        generation::validate_row_index(row_index).map_err(ExcelError::from)?;
        Ok(XlsxRow {
            worksheet: self.worksheet,
            row_index,
        })
    }
}

