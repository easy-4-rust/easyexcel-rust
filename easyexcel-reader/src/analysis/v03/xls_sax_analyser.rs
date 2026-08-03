//! 对应 Java：`com.alibaba.excel.analysis.v03.XlsSaxAnalyser`.

use std::path::{Path, PathBuf};

use easyexcel_core::{AnalysisContext, ExcelError, ExcelRow, ReadListener, Result};

use crate::analysis::excel_read_executor::ExcelReadExecutor;
use crate::analysis::v03::biff_record_stream::{read_workbook_stream, walk_biff_records};
use crate::analysis::v03::xls_list_sheet_listener::XlsListSheetListener;
use crate::analysis::v03::xls_record_dispatcher::{XlsRecordDispatchState, XlsRecordDispatcher};
use crate::context::{DefaultXlsReadContext, ReadSheet, XlsReadContext};
use crate::{ReadOptions, read_xls};

/// 对应 Java：`XlsSaxAnalyser implements HSSFListener, ExcelReadExecutor`.
///
/// Java registers 19 BIFF record handlers and drives POI `HSSFEventFactory`.
/// Rust runs the same SID dispatch over the real OLE Workbook stream, then uses
/// [`read_xls`] (calamine) for typed row materialisation.
pub struct XlsSaxAnalyser {
    /// Workbook path. (Java `ReadWorkbookHolder.file`)
    path: PathBuf,
    /// Read options collapsed from Java holders.
    options: ReadOptions,
    /// XLS read context. (Java `xlsReadContext`)
    xls_read_context: DefaultXlsReadContext,
    /// Discovered worksheets. (Java `sheetList`)
    sheet_list: Vec<ReadSheet>,
    /// Captures errors from the void [`ExcelReadExecutor::execute`] entry.
    last_error: Option<ExcelError>,
    /// Java-compatible BIFF SID handler registry and observable state.
    record_dispatcher: XlsRecordDispatcher,
}

impl XlsSaxAnalyser {
    /// 对应 Java：`XlsSaxAnalyser(XlsReadContext)`.
    ///
    /// Sheet discovery uses calamine metadata, equivalent to Java
    /// `XlsListSheetListener.execute()` pre-scan.
    ///
    /// # Errors
    ///
    /// Returns when the workbook cannot be opened or contains no sheets.
    pub fn new(
        mut xls_read_context: DefaultXlsReadContext,
        path: impl Into<PathBuf>,
        options: ReadOptions,
    ) -> Result<Self> {
        let path = path.into();
        let sheet_list = {
            let mut listener =
                XlsListSheetListener::new(&mut xls_read_context, &path, options.clone());
            listener.execute()?.to_vec()
        };
        if sheet_list.is_empty() {
            return Err(ExcelError::Format("Can not find any sheet!".to_owned()));
        }
        let record_dispatcher = XlsRecordDispatcher::new(&options);
        Ok(Self {
            path,
            options,
            xls_read_context,
            sheet_list,
            last_error: None,
            record_dispatcher,
        })
    }

    /// Convenience constructor mirroring Java `ExcelAnalyserImpl` wiring.
    ///
    /// # Errors
    ///
    /// Propagates [`Self::new`] failures.
    pub fn from_path(path: impl Into<PathBuf>, options: ReadOptions) -> Result<Self> {
        let context = DefaultXlsReadContext::new(&options);
        Self::new(context, path, options)
    }

    /// Returns the bound workbook path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the XLS read context. (Java `xlsReadContext` field)
    #[must_use]
    pub fn xls_read_context(&self) -> &DefaultXlsReadContext {
        &self.xls_read_context
    }

    /// Returns the last error recorded by the void [`ExcelReadExecutor::execute`] entry.
    #[must_use]
    pub const fn last_error(&self) -> Option<&ExcelError> {
        self.last_error.as_ref()
    }

    /// 对应 Java：`processRecord(Record)` from `HSSFListener`.
    ///
    /// # Errors
    ///
    /// Routes the record to the registered handler. Unknown SIDs are ignored,
    /// matching Java's `XLS_RECORD_HANDLER_MAP.get(...) == null` branch.
    pub fn process_record(&mut self, record_sid: u16, data: &[u8]) -> Result<()> {
        self.record_dispatcher.process_record(record_sid, data)
    }

    /// Returns state produced by the real BIFF handler dispatch.
    #[must_use]
    pub const fn record_dispatch_state(&self) -> &XlsRecordDispatchState {
        self.record_dispatcher.state()
    }

    fn dispatch_workbook_records(&mut self) -> Result<()> {
        let workbook = read_workbook_stream(&self.path)?;
        self.record_dispatcher.reset();
        walk_biff_records(&workbook, |record_sid, data| {
            self.process_record(record_sid, data)
        })?;
        self.record_dispatcher.finish_records()
    }

    /// Typed execute path. (Java `execute()` + listener on `ReadWorkbook`)
    ///
    /// # Errors
    ///
    /// Propagates workbook, sheet-selection, conversion, or listener errors.
    pub fn execute_with_listener<T, L>(&mut self, listener: &mut L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        let options = self.options.clone();
        self.execute::<T, L>(&options, listener)
    }

    fn execute_with_options<T, L>(&mut self, options: &ReadOptions, listener: &mut L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        let result = self
            .dispatch_workbook_records()
            .and_then(|()| read_xls::<T, L>(&self.path, options, listener));
        match result {
            Ok(()) => {
                self.last_error = None;
                Ok(())
            }
            Err(error) => {
                self.last_error = Some(error.clone());
                Err(error)
            }
        }
    }

    /// Returns the listener callback context from the embedded read context.
    #[must_use]
    pub fn analysis_context(&self) -> &AnalysisContext {
        self.xls_read_context
            .analysis_context_impl()
            .analysis_context()
    }
}

impl ExcelReadExecutor for XlsSaxAnalyser {
    /// 对应 Java：`sheetList()`.
    fn sheet_list(&self) -> &[ReadSheet] {
        &self.sheet_list
    }

    fn execute<T, L>(&mut self, options: &ReadOptions, listener: &mut L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        self.execute_with_options::<T, L>(options, listener)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;

    use base64::Engine;
    use easyexcel_core::DynamicRow;
    use flate2::read::GzDecoder;
    use tempfile::{NamedTempFile, tempdir};

    use super::*;

    #[derive(Default)]
    struct CollectingListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for CollectingListener {
        fn invoke(&mut self, data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }

        fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> Result<()> {
            Ok(())
        }
    }

    /// Materializes the embedded Java multisheet `.xls` fixture for unit tests.
    fn write_java_multisheet_xls() -> NamedTempFile {
        let file = NamedTempFile::with_suffix(".xls").expect("temp xls");
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(include_str!("../../fixtures/java-multiplesheets.xls.gz.b64").trim())
            .expect("fixture b64");
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut workbook = Vec::new();
        decoder.read_to_end(&mut workbook).expect("gunzip");
        fs::write(file.path(), workbook).expect("write xls");
        file
    }

    #[test]
    fn sheet_list_discovers_worksheet_names() -> Result<()> {
        let file = write_java_multisheet_xls();
        let options = ReadOptions::default();
        let analyser = XlsSaxAnalyser::from_path(file.path(), options)?;
        assert_eq!(analyser.sheet_list().len(), 6);
        assert!(!analyser.sheet_list()[0].sheet_name().is_empty());
        assert!(
            !analyser
                .xls_read_context()
                .xls_read_workbook_holder()
                .need_read_sheet()
        );
        assert_eq!(
            analyser
                .xls_read_context()
                .xls_read_workbook_holder()
                .inner()
                .actual_sheet_data_list()
                .expect("actual sheet list"),
            analyser.sheet_list()
        );
        Ok(())
    }

    #[test]
    fn execute_with_listener_delegates_to_read_xls() -> Result<()> {
        let file = write_java_multisheet_xls();
        let options = ReadOptions {
            head_row_number: 1,
            sheet: crate::SheetSelector::Index(0),
            ..ReadOptions::default()
        };
        let mut analyser = XlsSaxAnalyser::from_path(file.path(), options)?;
        let mut listener = CollectingListener::default();
        analyser.execute_with_listener::<DynamicRow, _>(&mut listener)?;
        assert!(!listener.rows.is_empty());
        Ok(())
    }

    #[test]
    fn trait_execute_runs_the_real_xls_parser() -> Result<()> {
        let file = write_java_multisheet_xls();
        let options = ReadOptions {
            head_row_number: 1,
            sheet: crate::SheetSelector::Index(0),
            ..ReadOptions::default()
        };
        let mut analyser =
            XlsSaxAnalyser::from_path(file.path(), options.clone()).expect("analyser");
        let mut listener = CollectingListener::default();
        ExcelReadExecutor::execute::<DynamicRow, _>(&mut analyser, &options, &mut listener)?;
        assert!(!listener.rows.is_empty());
        assert!(analyser.last_error().is_none());
        Ok(())
    }

    #[test]
    // 对应 Java：断言为 BIFF `f64` 位级往返值（`to_le_bytes`/`from_le_bytes` 无损），
    // 精确相等即预期语义，不做容差比较。
    #[allow(clippy::float_cmp)]
    fn process_record_dispatches_number_handler() -> Result<()> {
        let file = write_java_multisheet_xls();
        let mut analyser =
            XlsSaxAnalyser::from_path(file.path(), ReadOptions::default()).expect("analyser");
        let mut payload = vec![4, 0, 2, 0, 9, 0];
        payload.extend_from_slice(&12.5f64.to_le_bytes());
        analyser.process_record(0x0203, &payload)?;
        let cell = analyser
            .record_dispatch_state()
            .last_number_cell()
            .expect("real NumberRecordHandler output");
        assert_eq!((cell.row, cell.column, cell.format_index), (4, 2, 9));
        assert_eq!(cell.value, 12.5);
        Ok(())
    }

    #[test]
    fn execute_walks_real_workbook_biff_records() -> Result<()> {
        let file = write_java_multisheet_xls();
        let options = ReadOptions {
            head_row_number: 1,
            sheet: crate::SheetSelector::Index(0),
            ..ReadOptions::default()
        };
        let mut analyser = XlsSaxAnalyser::from_path(file.path(), options)?;
        let mut listener = CollectingListener::default();

        analyser.execute_with_listener::<DynamicRow, _>(&mut listener)?;

        let state = analyser.record_dispatch_state();
        assert!(state.total_record_count() > 0);
        assert!(state.handled_record_count() > 0);
        assert_eq!(state.bound_sheets().len(), analyser.sheet_list().len());
        assert_eq!(state.worksheet_bof_count(), analyser.sheet_list().len());
        assert!(state.workbook_bof_count() >= 1);
        assert!(state.eof_count() >= analyser.sheet_list().len());
        assert!(!state.shared_strings().is_empty());
        assert_eq!(
            state.unique_string_count(),
            u32::try_from(state.shared_strings().len()).ok()
        );
        assert!(
            state
                .bound_sheets()
                .iter()
                .all(|sheet| !sheet.name.is_empty())
        );
        Ok(())
    }

    #[test]
    fn list_xls_sheets_matches_analyser_sheet_list() -> Result<()> {
        let file = write_java_multisheet_xls();
        let options = ReadOptions::default();
        let discovered = crate::list_xls_sheets(file.path(), &options)?;
        let analyser = XlsSaxAnalyser::from_path(file.path(), options)?;
        assert_eq!(discovered.len(), analyser.sheet_list().len());
        Ok(())
    }

    #[test]
    fn empty_workbook_path_fails_sheet_discovery() {
        let directory = tempdir().expect("tempdir");
        let path = directory.path().join("missing.xls");
        assert!(crate::list_xls_sheets(&path, &ReadOptions::default()).is_err());
    }
}

#[cfg(test)]
mod tests_extra {
    use std::fs;
    use std::io::Read;

    use base64::Engine;
    use easyexcel_core::DynamicRow;
    use flate2::read::GzDecoder;
    use tempfile::NamedTempFile;

    use super::*;

    #[derive(Default)]
    struct ExtraCollectingListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for ExtraCollectingListener {
        fn invoke(&mut self, data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }
    }

    struct FailingListener;

    impl ReadListener<DynamicRow> for FailingListener {
        fn invoke(&mut self, _data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            Err(ExcelError::Format("listener failed".to_owned()))
        }

        fn invoke_head(
            &mut self,
            _head: &std::collections::HashMap<String, usize>,
            _context: &AnalysisContext,
        ) -> Result<()> {
            Err(ExcelError::Format("listener failed".to_owned()))
        }
    }

    fn write_java_multisheet_xls() -> NamedTempFile {
        let file = NamedTempFile::with_suffix(".xls").expect("temp xls");
        let compressed = base64::engine::general_purpose::STANDARD
            .decode(include_str!("../../fixtures/java-multiplesheets.xls.gz.b64").trim())
            .expect("fixture b64");
        let mut decoder = GzDecoder::new(compressed.as_slice());
        let mut workbook = Vec::new();
        decoder.read_to_end(&mut workbook).expect("gunzip");
        fs::write(file.path(), workbook).expect("write xls");
        file
    }

    #[test]
    fn accessors_and_error_recording() -> Result<()> {
        // 对应 Java：XlsSaxAnalyser 公开访问器与 lastError 记录
        let file = write_java_multisheet_xls();
        let options = ReadOptions {
            head_row_number: 1,
            sheet: crate::SheetSelector::Index(0),
            ..ReadOptions::default()
        };
        let mut analyser = XlsSaxAnalyser::from_path(file.path(), options)?;

        assert_eq!(analyser.path(), file.path());
        assert!(analyser.analysis_context().sheet_name().is_empty());
        assert!(analyser.last_error().is_none());

        // 监听器失败 → execute 失败并记录 last_error
        let mut failing = FailingListener;
        assert!(
            analyser
                .execute_with_listener::<DynamicRow, _>(&mut failing)
                .is_err()
        );
        assert!(analyser.last_error().is_some());

        // 成功执行后清除 last_error
        let mut collecting = ExtraCollectingListener::default();
        analyser.execute_with_listener::<DynamicRow, _>(&mut collecting)?;
        assert!(!collecting.rows.is_empty());
        assert!(analyser.last_error().is_none());
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra2 {
    use std::fs;
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    /// 构造一个 BIFF 流中只有 BOF/EOF、没有任何 `BoundSheet` 的 OLE2 文档。
    /// calamine 能打开但 sheet 列表为空，从而触发“Can not find any sheet”。
    // 对应 Java：BIFF 记录长度字段为 u16，测试 payload 固定（≤4 字节），
    // `as u16` 不可能截断。
    #[allow(clippy::cast_possible_truncation)]
    fn write_zero_sheet_xls() -> NamedTempFile {
        let file = NamedTempFile::with_suffix(".xls").expect("temp xls");
        let directory = file.path().parent().expect("tempdir parent");
        let compound_path = directory.join(format!(
            "zero-sheet-{}.xls",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        {
            let mut compound = cfb::create(&compound_path).expect("cfb create");
            let mut stream = compound
                .create_stream("/Workbook")
                .expect("Workbook stream");
            // BOF (0x0809) + EOF (0x000A)，无 BoundSheet → 0 个工作表
            let mut records = Vec::new();
            for (sid, payload) in [(0x0809u16, vec![0u8, 0, 0x05, 0x00]), (0x000A, vec![])] {
                records.extend_from_slice(&sid.to_le_bytes());
                records.extend_from_slice(&(payload.len() as u16).to_le_bytes());
                records.extend_from_slice(&payload);
            }
            stream.write_all(&records).expect("write records");
            drop(stream);
            compound.flush().expect("flush");
        }
        fs::copy(&compound_path, file.path()).expect("copy cfb into temp file");
        let _ = fs::remove_file(&compound_path);
        file
    }

    #[test]
    // 对应 Java：BIFF 记录长度字段为 u16，测试 payload 均 ≤ 4 字节，
    // `as u16` 截断不可能发生（fixture 固定长度）。
    #[allow(clippy::cast_possible_truncation)]
    fn new_rejects_workbooks_without_any_sheet() {
        // 对应 Java：XlsSaxAnalyser 构造时 sheetList 为空报错
        let file = write_zero_sheet_xls();
        let Err(error) = XlsSaxAnalyser::from_path(file.path(), ReadOptions::default()) else {
            panic!("zero-sheet workbook must fail");
        };
        assert!(error.to_string().contains("Can not find any sheet"));
    }

    #[test]
    fn from_path_errors_on_missing_xls_file() {
        // 对应 Java：无法打开的工作簿报错
        let directory = tempfile::tempdir().expect("tempdir");
        let missing = directory.path().join("missing.xls");
        assert!(XlsSaxAnalyser::from_path(&missing, ReadOptions::default()).is_err());
    }
}
