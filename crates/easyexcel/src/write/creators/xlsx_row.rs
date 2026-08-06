/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct XlsxRow<'a> {
    pub(crate) worksheet: &'a mut Worksheet,
    pub(crate) row_index: u32,
}

impl CellCreator for XlsxRow<'_> {
    type Cell<'a>
        = XlsxCell<'a>
    where
        Self: 'a;

    fn create_cell(&mut self, column_index: u16) -> Result<Self::Cell<'_>> {
        generation::validate_column_index(column_index).map_err(ExcelError::from)?;
        Ok(XlsxCell {
            worksheet: self.worksheet,
            row_index: self.row_index,
            column_index,
        })
    }
}

