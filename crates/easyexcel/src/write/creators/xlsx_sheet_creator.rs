/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct XlsxSheetCreator<'a> {
    pub(crate) workbook: &'a mut Workbook,
    pub(crate) constant_memory: bool,
}

impl SheetCreator for XlsxSheetCreator<'_> {
    type Sheet<'a>
        = &'a mut Worksheet
    where
        Self: 'a;

    fn create_sheet(&mut self, sheet_name: &str) -> Result<Self::Sheet<'_>> {
        generation::create_worksheet(self.workbook, sheet_name, self.constant_memory)
            .map_err(format_error)
    }
}

