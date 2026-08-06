/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct XlsxWorkBookCreator;

impl WorkBookCreator for XlsxWorkBookCreator {
    type WorkBook = Workbook;

    fn create_work_book(self) -> Result<Self::WorkBook> {
        Ok(easyexcel_xlsx::xlsx::generation::new_workbook())
    }
}

