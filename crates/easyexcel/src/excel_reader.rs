//! 对应 Java：`com.alibaba.excel.ExcelReader`.

use std::marker::PhantomData;
use std::path::PathBuf;

use easyexcel_io::io::file_utils::TemporaryInput;

use crate::core::{
    AnalysisContext, CompositeReadListener, ExcelError, ExcelRow, ReadListener, Result,
};
use crate::support::ExcelTypeEnum;

use crate::analysis::excel_analyser::ExcelAnalyser;
use crate::analysis::excel_analyser_impl::ExcelAnalyserImpl;
use crate::analysis::excel_read_executor::ExcelReadExecutorKind;
use crate::context::read_sheet::ReadSheet;
use crate::{ReadOptions, SheetSelector};

/// Event-driven workbook reader.
///
/// 对应 Java：`com.alibaba.excel.ExcelReader`.
pub struct ExcelReader<T, L> {
    analyser: ExcelAnalyserImpl,
    listener: Option<L>,
    marker: PhantomData<T>,
}

impl<T, L> ExcelReader<T, L>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    /// Creates a reader bound to a workbook path and options.
    ///
    /// 对应 Java：`ExcelReader(ReadWorkbook)`.
    ///
    /// # Errors
    ///
    /// 当工作簿无法打开或解析（路径不存在、文件损坏等）时返回 [`ExcelError`]。
    pub fn new(path: impl Into<PathBuf>, options: ReadOptions, listener: L) -> Result<Self> {
        Ok(Self {
            analyser: ExcelAnalyserImpl::from_path(path, options)?,
            listener: Some(listener),
            marker: PhantomData,
        })
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Creates a reader for a path owned by a temporary-input guard.
    ///
    /// The compatible builder uses this for Java `read(InputStream, ...)`.
    pub(crate) fn from_temporary_input(
        path: impl Into<PathBuf>,
        temporary_input: std::sync::Arc<TemporaryInput>,
        options: ReadOptions,
        listener: L,
    ) -> Result<Self> {
        Ok(Self {
            analyser: ExcelAnalyserImpl::from_temporary_input(path, temporary_input, options)?,
            listener: Some(listener),
            marker: PhantomData,
        })
    }

    /// Returns whether this reader owns a materialised input-stream guard.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.ExcelReader。
    pub const fn has_temporary_input(&self) -> bool {
        self.analyser.has_temporary_input()
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Parses every configured worksheet. (Java `readAll()`)
    ///
    /// # Errors
    ///
    /// 当任一工作表的 SAX/记录解析失败，或读取已完成（`finish()` 之后）时返回
    /// [`ExcelError`]。
    ///
    /// # Panics
    ///
    /// 内部 listener 被 `into_listener` 提前取走而调用方仍继续读取时会 panic。
    pub fn read_all(&mut self) -> Result<()> {
        let listener = self
            .listener
            .as_mut()
            .expect("ExcelReader listener is present until into_listener");
        ExcelAnalyser::analysis_with_listener::<T, L>(&mut self.analyser, listener)
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Deprecated Java `read()` alias for [`Self::read_all`].
    #[deprecated(note = "please use read_all()")]
    ///
    /// # Errors
    ///
    /// 与 [`Self::read_all`] 相同：解析失败时返回 [`ExcelError`]。
    pub fn read_deprecated(&mut self) -> Result<()> {
        self.read_all()
    }
    /// 对应 Java：com.alibaba.excel.ExcelReader。
    pub(crate) fn read_all_with_additional_listener<M>(&mut self, listener: &mut M) -> Result<()>
    where
        T: Clone,
        M: ReadListener<T>,
    {
        let primary = self
            .listener
            .as_mut()
            .expect("ExcelReader listener is present until into_listener");
        let mut listeners = CompositeReadListener::new(primary, listener);
        ExcelAnalyser::analysis_with_listener::<T, _>(&mut self.analyser, &mut listeners)
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Parses the supplied worksheets. (Java `read(ReadSheet...)`)
    ///
    /// # Errors
    ///
    /// 当 `sheets` 为空、未解析出工作簿类型，或任一工作表的解析失败时返回
    /// [`ExcelError`]。
    ///
    /// # Panics
    ///
    /// 内部 listener 被 `into_listener` 提前取走而调用方仍继续读取时会 panic。
    pub fn read(&mut self, sheets: &[ReadSheet]) -> Result<&mut Self> {
        let listener = self
            .listener
            .as_mut()
            .expect("ExcelReader listener is present until into_listener");
        Self::read_sheets_with_listener(&mut self.analyser, listener, sheets)?;
        Ok(self)
    }
    /// 对应 Java：com.alibaba.excel.ExcelReader。
    pub(crate) fn read_with_additional_listener<M>(
        &mut self,
        sheets: &[ReadSheet],
        listener: &mut M,
    ) -> Result<&mut Self>
    where
        T: Clone,
        M: ReadListener<T>,
    {
        let primary = self
            .listener
            .as_mut()
            .expect("ExcelReader listener is present until into_listener");
        let mut listeners = CompositeReadListener::new(primary, listener);
        Self::read_sheets_with_listener(&mut self.analyser, &mut listeners, sheets)?;
        Ok(self)
    }

    fn read_sheets_with_listener<M>(
        analyser: &mut ExcelAnalyserImpl,
        listener: &mut M,
        sheets: &[ReadSheet],
    ) -> Result<()>
    where
        M: ReadListener<T>,
    {
        if sheets.is_empty() {
            return Err(ExcelError::Format(
                "Specify at least one read sheet.".to_owned(),
            ));
        }

        let workbook_head_row_number = analyser.options().head_row_number;
        let workbook_scientific_format = analyser.options().scientific_format;
        let path = analyser
            .path()
            .ok_or_else(|| ExcelError::Format("ExcelReader has no workbook path".to_owned()))?;
        let actual_sheets = match analyser.excel_type() {
            Some(ExcelTypeEnum::Xlsx) => crate::read::list_xlsx_sheets(path, analyser.options())?,
            Some(ExcelTypeEnum::Xls) => crate::read::list_xls_sheets(path, analyser.options())?,
            Some(ExcelTypeEnum::Csv) => vec![(0, "Sheet1".to_owned())],
            None => {
                return Err(ExcelError::Format(
                    "ExcelReader has no resolved workbook type".to_owned(),
                ));
            }
        };

        // Java executors enumerate actual workbook sheets and use the first
        // matching parameter sheet. This preserves workbook order, ignores
        // duplicate parameters, and leaves unknown selections unread.
        for (actual_sheet_no, actual_sheet_name) in actual_sheets {
            let Some(sheet) = sheets.iter().find(|sheet| {
                (sheet.has_sheet_no() && sheet.sheet_no() == actual_sheet_no)
                    || (!sheet.sheet_name().is_empty()
                        && easyexcel_utils::string_utils::equals_with_optional_java_trim(
                            &actual_sheet_name,
                            sheet.sheet_name(),
                            analyser.options().auto_trim,
                        ))
            }) else {
                continue;
            };
            analyser.set_sheet_selector(SheetSelector::Index(actual_sheet_no));
            let options = analyser.options_mut();
            options.head_row_number = sheet.head_row_number().unwrap_or(workbook_head_row_number);
            options.scientific_format =
                sheet
                    .use_scientific_format()
                    .map_or(workbook_scientific_format, |enabled| {
                        if enabled {
                            crate::ScientificFormatMode::Scientific
                        } else {
                            crate::ScientificFormatMode::Plain
                        }
                    });
            ExcelAnalyser::analysis_with_listener::<T, _>(analyser, listener)?;
        }
        Ok(())
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Returns the live analysis context. (Java `analysisContext()`)
    #[must_use]
    pub fn analysis_context(&self) -> &AnalysisContext {
        ExcelAnalyser::analysis_context(&self.analyser)
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Deprecated Java `getAnalysisContext()` alias.
    #[deprecated(note = "please use analysis_context()")]
    #[must_use]
    pub fn get_analysis_context(&self) -> &AnalysisContext {
        self.analysis_context()
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Returns the selected XLSX/XLS/CSV executor.
    #[must_use]
    pub fn excel_executor(&self) -> &ExcelReadExecutorKind {
        ExcelAnalyser::excel_executor(&self.analyser)
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Completes the read and releases resources. (Java `finish()`)
    pub fn finish(&mut self) {
        ExcelAnalyser::finish(&mut self.analyser);
    }

    /// 对应 Java：com.alibaba.excel.ExcelReader。 Java `Closeable.close()` alias. Finishing is idempotent.
    pub fn close(&mut self) {
        self.finish();
    }

    /// Consumes the reader after finishing and returns its listener.
    ///
    /// This crate-internal handoff lets synchronous facade builders collect
    /// rows while still routing through the same analyser/executor lifecycle.
    pub(crate) fn into_listener(mut self) -> L {
        self.finish();
        self.listener
            .take()
            .expect("ExcelReader listener can only be taken once")
    }
}

impl<T, L> Drop for ExcelReader<T, L> {
    fn drop(&mut self) {
        ExcelAnalyser::finish(&mut self.analyser);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    use crate::core::DynamicRow;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::ReadOptions;
    #[derive(Default)]
    struct CollectListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for CollectListener {
        fn invoke(&mut self, data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct SheetTraceListener {
        sheets: Arc<Mutex<Vec<String>>>,
    }

    impl ReadListener<DynamicRow> for SheetTraceListener {
        fn invoke(&mut self, _data: DynamicRow, context: &AnalysisContext) -> Result<()> {
            self.sheets
                .lock()
                .expect("sheet trace lock")
                .push(context.sheet_name().to_owned());
            Ok(())
        }
    }

    fn multi_sheet_workbook() -> Result<NamedTempFile> {
        let file = NamedTempFile::with_suffix(".xlsx")?;
        let mut workbook = Workbook::new();
        let first = workbook.add_worksheet();
        first
            .set_name("First")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        first
            .write_string(0, 0, "Value")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        first
            .write_string(1, 0, "one")
            .map_err(|error| ExcelError::Format(error.to_string()))?;

        let second = workbook.add_worksheet();
        second
            .set_name("Second")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        second
            .write_string(0, 0, "ignored heading")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        second
            .write_string(1, 0, "Value")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        second
            .write_string(2, 0, "two")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        workbook
            .save(file.path())
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        Ok(file)
    }

    #[test]
    fn excel_reader_read_all_loads_csv_rows() -> Result<()> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "name,age")?;
        writeln!(file, "reader,30")?;
        let listener = CollectListener::default();
        let mut reader = ExcelReader::new(file.path(), ReadOptions::default(), listener)?;
        reader.read_all()?;
        Ok(())
    }

    #[test]
    #[allow(deprecated)]
    fn excel_reader_exposes_the_real_executor_and_java_lifecycle_aliases() -> Result<()> {
        let file = multi_sheet_workbook()?;
        let listener = SheetTraceListener::default();
        let mut reader = ExcelReader::new(file.path(), ReadOptions::default(), listener)?;

        let sheets = reader.excel_executor().sheet_list();
        assert_eq!(sheets.len(), 2);
        assert_eq!(sheets[0].sheet_name(), "First");
        assert_eq!(sheets[1].sheet_name(), "Second");
        assert!(std::ptr::eq(
            reader.analysis_context(),
            reader.get_analysis_context()
        ));

        reader.read_deprecated()?;
        reader.close();
        reader.close();
        let error = reader
            .read_all()
            .expect_err("a closed reader must reject another analysis");
        assert!(error.to_string().contains("called after finish"));
        Ok(())
    }

    #[test]
    fn excel_reader_csv_executor_reports_its_real_logical_sheet() -> Result<()> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "value")?;
        let reader = ExcelReader::new(
            file.path(),
            ReadOptions::default(),
            CollectListener::default(),
        )?;
        let sheets = reader.excel_executor().sheet_list();
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheets[0].sheet_no(), 0);
        assert_eq!(sheets[0].sheet_name(), "Sheet1");
        Ok(())
    }

    #[test]
    fn excel_reader_read_rejects_an_empty_sheet_list_like_java() -> Result<()> {
        let file = multi_sheet_workbook()?;
        let listener = SheetTraceListener::default();
        let mut reader = ExcelReader::new(file.path(), ReadOptions::default(), listener)?;
        let Err(error) = reader.read(&[]) else {
            panic!("empty sheet list must fail");
        };
        assert_eq!(
            error.to_string(),
            "excel format error: Specify at least one read sheet."
        );
        Ok(())
    }

    #[test]
    fn excel_reader_read_processes_each_sheet_and_applies_sheet_parameters() -> Result<()> {
        let file = multi_sheet_workbook()?;
        let listener = SheetTraceListener::default();
        let observed = Arc::clone(&listener.sheets);
        let mut reader = ExcelReader::new(file.path(), ReadOptions::default(), listener)?;
        let first = ReadSheet::new(0);
        let mut second = ReadSheet::named("Second");
        second.set_head_row_number(2);

        // Java enumerates actual workbook sheets, not parameter-list order.
        reader.read(&[second, first])?;

        assert_eq!(
            *observed.lock().expect("sheet trace lock"),
            vec!["First".to_owned(), "Second".to_owned()]
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra2 {
    use std::fs;
    use std::io::Read;

    use crate::core::DynamicRow;
    use base64::Engine;
    use flate2::read::GzDecoder;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;

    use super::*;
    use crate::ReadOptions;

    #[derive(Default)]
    struct ExtraCollectListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for ExtraCollectListener {
        fn invoke(&mut self, data: DynamicRow, _context: &AnalysisContext) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }
    }

    fn two_sheet_workbook() -> Result<NamedTempFile> {
        let file = NamedTempFile::with_suffix(".xlsx")?;
        let mut workbook = Workbook::new();
        let first = workbook.add_worksheet();
        first
            .set_name("First")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        first
            .write_string(0, 0, "Value")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        first
            .write_string(1, 0, "one")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        let second = workbook.add_worksheet();
        second
            .set_name("Second")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        second
            .write_string(0, 0, "Value")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        second
            .write_string(1, 0, "two")
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        workbook
            .save(file.path())
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        Ok(file)
    }

    /// 物化 Java 官方多 sheet .xls fixture。
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
    fn read_processes_xls_workbook_sheets() -> Result<()> {
        // 对应 Java：ExcelReader.read(ReadSheet...) 走 XlsSaxAnalyser 分支
        let file = write_java_multisheet_xls();
        let options = ReadOptions {
            head_row_number: 1,
            ..ReadOptions::default()
        };
        let listener = ExtraCollectListener::default();
        let mut reader = ExcelReader::new(file.path(), options, listener)?;
        let mut sheet = ReadSheet::new(0);
        sheet.set_head_row_number(1);
        reader.read(&[sheet])?;
        Ok(())
    }

    #[test]
    fn read_skips_selections_that_match_no_actual_sheet() -> Result<()> {
        // 对应 Java：找不到匹配的实际工作表时静默跳过
        let file = two_sheet_workbook()?;
        let listener = ExtraCollectListener::default();
        let mut reader = ExcelReader::new(file.path(), ReadOptions::default(), listener)?;
        reader.read(&[ReadSheet::new(9), ReadSheet::named("Nope")])?;
        assert!(reader.analysis_context().sheet_name().is_empty());
        Ok(())
    }

    #[test]
    fn read_applies_sheet_scientific_format_override() -> Result<()> {
        // 对应 Java：ReadSheet.useScientificFormat 覆盖工作簿级配置
        let file = two_sheet_workbook()?;
        let options = ReadOptions {
            scientific_format: crate::ScientificFormatMode::Plain,
            ..ReadOptions::default()
        };
        let mut reader = ExcelReader::new(file.path(), options, ExtraCollectListener::default())?;

        let mut scientific = ReadSheet::new(0);
        scientific.set_head_row_number(1);
        scientific.set_use_scientific_format(true);
        reader.read(&[scientific])?;
        assert_eq!(
            reader.analyser.options().scientific_format,
            crate::ScientificFormatMode::Scientific
        );

        let mut plain = ReadSheet::new(1);
        plain.set_head_row_number(1);
        plain.set_use_scientific_format(false);
        reader.read(&[plain])?;
        assert_eq!(
            reader.analyser.options().scientific_format,
            crate::ScientificFormatMode::Plain
        );
        Ok(())
    }

    #[test]
    fn read_with_additional_listener_runs_both_listeners() -> Result<()> {
        // 对应 Java：read 时附加监听器与原监听器同时收到回调
        let file = two_sheet_workbook()?;
        let mut reader = ExcelReader::new(
            file.path(),
            ReadOptions::default(),
            ExtraCollectListener::default(),
        )?;
        let mut additional = ExtraCollectListener::default();
        reader.read_with_additional_listener(&[ReadSheet::new(0)], &mut additional)?;
        assert!(!additional.rows.is_empty());
        Ok(())
    }
}
