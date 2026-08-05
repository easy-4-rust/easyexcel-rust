//! 对应 Java：`com.alibaba.excel.analysis.v03.XlsListSheetListener`.

use std::path::{Path, PathBuf};

use crate::core::Result;

use crate::ReadOptions;
use crate::context::{DefaultXlsReadContext, ReadSheet};
use crate::read::list_xls_sheets;

/// 对应 Java：`XlsListSheetListener implements HSSFListener`.
///
/// Java's listener pre-scans BIFF records to enumerate sheet names
/// before the main read. Rust performs the same metadata-only pass through
/// calamine and stores the result in `actual_sheet_data_list`.
pub struct XlsListSheetListener<'a> {
    xls_read_context: &'a mut DefaultXlsReadContext,
    path: PathBuf,
    options: ReadOptions,
    sheet_list: Vec<ReadSheet>,
}

impl<'a> XlsListSheetListener<'a> {
    /// Creates the metadata-only listener.
    ///
    /// 对应 Java：`XlsListSheetListener(XlsReadContext)`, including
    /// `needReadSheet = false`.
    pub fn new(
        xls_read_context: &'a mut DefaultXlsReadContext,
        path: impl Into<PathBuf>,
        options: ReadOptions,
    ) -> Self {
        xls_read_context
            .xls_read_workbook_holder_mut()
            .set_need_read_sheet(false);
        Self {
            xls_read_context,
            path: path.into(),
            options,
            sheet_list: Vec::new(),
        }
    }

    /// Executes the real XLS metadata scan and stores discovered sheets.
    ///
    /// # Errors
    ///
    /// Propagates XLS open or metadata parsing errors.
    pub fn execute(&mut self) -> Result<&[ReadSheet]> {
        self.sheet_list = list_xls_sheets(&self.path, &self.options)?
            .into_iter()
            .map(|(sheet_no, sheet_name)| ReadSheet::with_name(sheet_no, sheet_name))
            .collect();
        self.xls_read_context
            .xls_read_workbook_holder_mut()
            .inner_mut()
            .set_actual_sheet_data_list(self.sheet_list.clone());
        Ok(&self.sheet_list)
    }

    /// Returns the bound XLS path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the last successfully discovered sheet list.
    #[must_use]
    pub fn sheet_list(&self) -> &[ReadSheet] {
        &self.sheet_list
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;

    use base64::Engine;
    use flate2::read::GzDecoder;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::context::{DefaultXlsReadContext, XlsReadContext};

    /// Materializes the embedded Java multisheet `.xls` fixture for unit tests.
    fn write_java_multisheet_xls() -> NamedTempFile {
        let file = NamedTempFile::with_suffix(".xls").expect("temp xls");
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(include_str!("../../../../easyexcel-test/tests/fixtures/java-fixtures/java-multiplesheets.xls.gz.b64").trim())
            .expect("fixture b64");
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut workbook = Vec::new();
        decoder.read_to_end(&mut workbook).expect("gunzip");
        fs::write(file.path(), workbook).expect("write xls");
        file
    }

    #[test]
    fn execute_scans_sheets_and_exposes_accessors() -> Result<()> {
        // 对应 Java：XlsListSheetListener.execute() 元数据扫描
        let file = write_java_multisheet_xls();
        let options = ReadOptions::default();
        let mut context = DefaultXlsReadContext::new(&options);
        let mut listener = XlsListSheetListener::new(&mut context, file.path(), options);

        assert_eq!(listener.path(), file.path());
        assert!(listener.sheet_list().is_empty());

        let sheets = listener.execute()?.to_vec();
        assert!(!sheets.is_empty());
        let discovered = listener.sheet_list().to_vec();
        assert_eq!(discovered, sheets);
        assert_eq!(discovered[0].sheet_no(), 0);
        assert!(!discovered[0].sheet_name().is_empty());

        // new() 设置 needReadSheet=false（对应 Java 构造行为）
        assert!(!context.xls_read_workbook_holder().need_read_sheet());
        Ok(())
    }
}
