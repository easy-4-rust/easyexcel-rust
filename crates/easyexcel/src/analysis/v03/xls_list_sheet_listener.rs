//! 对应 Java：`com.alibaba.excel.analysis.v03.XlsListSheetListener`.

use std::path::{Path, PathBuf};

use crate::core::{ExcelError, Result};

use crate::ReadOptions;
use crate::analysis::v03::handlers::bof_record_handler::BOF_SID;
use crate::analysis::v03::handlers::bound_sheet_record_handler::{
    BOUND_SHEET_SID, BoundSheetRecordHandler,
};
use crate::analysis::v03::xls_record_handler::XlsRecordHandler;
use crate::context::{DefaultXlsReadContext, ReadSheet};
use crate::read::list_xls_sheets;

/// 对应 Java：`XlsListSheetListener implements HSSFListener`.
///
/// Java's listener pre-scans BIFF records to enumerate sheet names
/// before the main read. Rust performs the same metadata-only pass through
/// `easyexcel-xls` and stores the result in `actual_sheet_data_list`.
pub struct XlsListSheetListener<'a> {
    xls_read_context: &'a mut DefaultXlsReadContext,
    path: PathBuf,
    options: ReadOptions,
    sheet_list: Vec<ReadSheet>,
    bound_sheet_handler: BoundSheetRecordHandler,
}

impl<'a> XlsListSheetListener<'a> {
    /// Creates the metadata-only listener.
    ///
    /// 对应 Java：`XlsListSheetListener(XlsReadContext)`, including
    /// `needReadSheet = false`.
    pub fn new(xls_read_context: &'a mut DefaultXlsReadContext) -> Self {
        let path = xls_read_context.file().map(Path::to_path_buf);
        let options = xls_read_context.options().clone();
        xls_read_context
            .xls_read_workbook_holder_mut()
            .set_need_read_sheet(false);
        Self {
            xls_read_context,
            path: path.unwrap_or_default(),
            options,
            sheet_list: Vec::new(),
            bound_sheet_handler: BoundSheetRecordHandler::new(),
        }
    }

    /// 使用显式路径创建元数据监听器。
    ///
    /// 这是 Rust 路径型扩展；Java 形状请使用 [`Self::new`]。
    #[must_use]
    pub fn from_path(
        xls_read_context: &'a mut DefaultXlsReadContext,
        path: impl Into<PathBuf>,
        options: ReadOptions,
    ) -> Self {
        xls_read_context.bind_input(path, options);
        Self::new(xls_read_context)
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.XlsListSheetListener。 Executes the real XLS metadata scan and stores discovered sheets.
    ///
    /// # Errors
    ///
    /// Propagates XLS open or metadata parsing errors.
    pub fn execute(&mut self) -> Result<()> {
        if self.path.as_os_str().is_empty() {
            return Err(ExcelError::Format(
                "XlsListSheetListener requires ReadWorkbook.file".to_owned(),
            ));
        }
        self.sheet_list = list_xls_sheets(&self.path, &self.options)?
            .into_iter()
            .map(|(sheet_no, sheet_name)| ReadSheet::with_name(sheet_no, sheet_name))
            .collect();
        self.xls_read_context
            .xls_read_workbook_holder_mut()
            .inner_mut()
            .set_actual_sheet_data_list(self.sheet_list.clone());
        Err(ExcelError::AnalysisStop(
            "Just need to get the list of sheets.".to_owned(),
        ))
    }

    /// 处理一个 BIFF8 record。
    ///
    /// 对应 Java：`XlsListSheetListener#processRecord(Record)`。只响应
    /// `BOUNDSHEET` 与 worksheet `BOF`，其他 SID 保持静默忽略。
    pub fn process_record(&mut self, record_sid: u16, data: &[u8]) {
        if record_sid == BOUND_SHEET_SID {
            self.bound_sheet_handler.process_record(record_sid, data);
            return;
        }
        if record_sid != BOF_SID
            || !matches!(
                easyexcel_xls::biff8::event_record::decode_bof_type(data),
                Some(easyexcel_xls::biff8::event_record::Biff8BofType::Worksheet)
            )
        {
            return;
        }
        self.sheet_list = self
            .bound_sheet_handler
            .ordered_sheets()
            .into_iter()
            .enumerate()
            .map(|(sheet_no, sheet)| ReadSheet::with_name(sheet_no, sheet.name))
            .collect();
        self.xls_read_context
            .xls_read_workbook_holder_mut()
            .inner_mut()
            .set_actual_sheet_data_list(self.sheet_list.clone());
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.XlsListSheetListener。 Returns the bound XLS path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 对应 Java：com.alibaba.excel.analysis.v03.XlsListSheetListener。 Returns the last successfully discovered sheet list.
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
            .decode(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/easyexcel-test/tests/fixtures/java-fixtures/java-multiplesheets.xls.gz.b64")).trim())
            .expect("fixture b64");
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut workbook = Vec::new();
        decoder.read_to_end(&mut workbook).expect("gunzip");
        fs::write(file.path(), workbook).expect("write xls");
        file
    }

    #[test]
    fn execute_scans_sheets_and_exposes_accessors() {
        // 对应 Java：XlsListSheetListener.execute() 元数据扫描
        let file = write_java_multisheet_xls();
        let options = ReadOptions::default();
        let mut context = DefaultXlsReadContext::new(&options);
        let mut listener = XlsListSheetListener::from_path(&mut context, file.path(), options);

        assert_eq!(listener.path(), file.path());
        assert!(listener.sheet_list().is_empty());

        let error = listener
            .execute()
            .expect_err("Java execute stops after discovering the sheet list");
        assert!(matches!(error, ExcelError::AnalysisStop(_)));
        let sheets = listener.sheet_list().to_vec();
        assert!(!sheets.is_empty());
        let discovered = listener.sheet_list().to_vec();
        assert_eq!(discovered, sheets);
        assert_eq!(discovered[0].sheet_no(), 0);
        assert!(!discovered[0].sheet_name().is_empty());

        // new() 设置 needReadSheet=false（对应 Java 构造行为）
        assert!(!context.xls_read_workbook_holder().need_read_sheet());
    }

    #[test]
    fn java_shaped_constructor_and_record_callback_publish_ordered_sheets() {
        let mut workbook = crate::read::metadata::ReadWorkbook::new();
        workbook.set_file("record-only.xls");
        let mut context = DefaultXlsReadContext::from_read_workbook(
            &workbook,
            crate::support::ExcelTypeEnum::Xls,
        );
        let mut listener = XlsListSheetListener::new(&mut context);

        let mut second = vec![200, 0, 0, 0, 0, 0, 1, 0];
        second.extend_from_slice(b"B");
        let mut first = vec![100, 0, 0, 0, 0, 0, 1, 0];
        first.extend_from_slice(b"A");
        listener.process_record(BOUND_SHEET_SID, &second);
        listener.process_record(0xFFFF, &[]);
        listener.process_record(BOUND_SHEET_SID, &first);
        listener.process_record(BOF_SID, &[0, 0, 0x10, 0]);

        assert_eq!(listener.path(), Path::new("record-only.xls"));
        assert_eq!(listener.sheet_list().len(), 2);
        assert_eq!(listener.sheet_list()[0].sheet_name(), "A");
        assert_eq!(listener.sheet_list()[1].sheet_name(), "B");
    }
}
