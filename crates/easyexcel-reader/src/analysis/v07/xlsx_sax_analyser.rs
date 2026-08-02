//! Mirrors Java `com.alibaba.excel.analysis.v07.XlsxSaxAnalyser`.

use std::path::{Path, PathBuf};

use easyexcel_core::{AnalysisContext, ExcelError, ExcelRow, ReadListener, Result};

use crate::analysis::excel_read_executor::ExcelReadExecutor;
use crate::context::{DefaultXlsxReadContext, ReadSheet, XlsxReadContext};
use crate::{ReadOptions, list_xlsx_sheets, read_xlsx};

/// Mirrors Java `XlsxSaxAnalyser implements ExcelReadExecutor`.
///
/// Java constructs OPCPackage, parses shared strings, and drives SAX through
/// `XlsxRowHandler`. Rust keeps the same public surface but delegates the
/// actual parse to [`read_xlsx`] on the quick-xml path.
pub struct XlsxSaxAnalyser {
    /// Workbook path. (Java `ReadWorkbookHolder.file` / temp file)
    path: PathBuf,
    /// Read options collapsed from Java holders.
    options: ReadOptions,
    /// XLSX read context. (Java `xlsxReadContext`)
    xlsx_read_context: DefaultXlsxReadContext,
    /// Discovered worksheets. (Java `sheetList`)
    sheet_list: Vec<ReadSheet>,
    /// Captures errors from the void [`ExcelReadExecutor::execute`] entry.
    last_error: Option<ExcelError>,
}

impl XlsxSaxAnalyser {
    /// Mirrors Java `XlsxSaxAnalyser(XlsxReadContext, InputStream decryptedStream)`.
    ///
    /// Sheet discovery uses the same quick-xml metadata path as [`read_xlsx`].
    /// Decryption is handled inside [`list_xlsx_sheets`] / [`read_xlsx`].
    ///
    /// # Errors
    ///
    /// Returns when the workbook cannot be opened or contains no sheets.
    pub fn new(
        xlsx_read_context: DefaultXlsxReadContext,
        path: impl Into<PathBuf>,
        options: ReadOptions,
    ) -> Result<Self> {
        let path = path.into();
        let discovered = list_xlsx_sheets(&path, &options)?;
        if discovered.is_empty() {
            return Err(ExcelError::Format("Can not find any sheet!".to_owned()));
        }
        let sheet_list = discovered
            .into_iter()
            .map(|(sheet_no, sheet_name)| ReadSheet::with_name(sheet_no, sheet_name))
            .collect();
        Ok(Self {
            path,
            options,
            xlsx_read_context,
            sheet_list,
            last_error: None,
        })
    }

    /// Convenience constructor mirroring Java `ExcelAnalyserImpl` wiring.
    ///
    /// # Errors
    ///
    /// Propagates [`Self::new`] failures.
    pub fn from_path(path: impl Into<PathBuf>, options: ReadOptions) -> Result<Self> {
        let context = DefaultXlsxReadContext::new(&options);
        Self::new(context, path, options)
    }

    /// Returns the bound workbook path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the XLSX read context. (Java `xlsxReadContext` field)
    #[must_use]
    pub fn xlsx_read_context(&self) -> &DefaultXlsxReadContext {
        &self.xlsx_read_context
    }

    /// Returns the last error recorded by the void [`ExcelReadExecutor::execute`] entry.
    #[must_use]
    pub const fn last_error(&self) -> Option<&ExcelError> {
        self.last_error.as_ref()
    }

    /// Mirrors Java `readComments(ReadSheet)` — comment replay after sheet SAX.
    ///
    /// # Errors
    ///
    /// Returns `ExcelError::Unsupported` because comment replay is already
    /// handled inside [`read_xlsx`] via worksheet extras.
    pub fn read_comments(&self, _read_sheet: &ReadSheet) -> Result<()> {
        Err(ExcelError::Unsupported(
            "XlsxSaxAnalyser.readComments is handled by read_xlsx extras dispatch".to_owned(),
        ))
    }

    /// Mirrors Java `parseXmlSource(InputStream, ContentHandler)`.
    ///
    /// # Errors
    ///
    /// Returns `ExcelError::Unsupported` — Rust routes XML through quick-xml
    /// handlers instead of Java SAX `ContentHandler`.
    pub fn parse_xml_source(&self) -> Result<()> {
        Err(ExcelError::Unsupported(
            "XlsxSaxAnalyser.parseXmlSource is internal to read_xlsx quick-xml handlers".to_owned(),
        ))
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
        let result = read_xlsx::<T, L>(&self.path, options, listener);
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
        self.xlsx_read_context
            .analysis_context_impl()
            .analysis_context()
    }
}

impl ExcelReadExecutor for XlsxSaxAnalyser {
    /// Mirrors Java `sheetList()`.
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
    use super::*;
    use easyexcel_core::DynamicRow;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;

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

    fn write_xlsx() -> NamedTempFile {
        let file = NamedTempFile::with_suffix(".xlsx").expect("temp xlsx");
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.write_string(0, 0, "name").expect("header");
        worksheet.write_string(1, 0, "alice").expect("row");
        workbook.save(file.path()).expect("save");
        file
    }

    #[test]
    fn sheet_list_discovers_worksheet_names() -> Result<()> {
        let file = write_xlsx();
        let options = ReadOptions::default();
        let analyser = XlsxSaxAnalyser::from_path(file.path(), options)?;
        assert_eq!(analyser.sheet_list().len(), 1);
        assert!(!analyser.sheet_list()[0].sheet_name().is_empty());
        Ok(())
    }

    #[test]
    fn execute_with_listener_delegates_to_read_xlsx() -> Result<()> {
        let file = write_xlsx();
        let mut options = ReadOptions::default();
        options.head_row_number = 1;
        let mut analyser = XlsxSaxAnalyser::from_path(file.path(), options)?;
        let mut listener = CollectingListener::default();
        analyser.execute_with_listener::<DynamicRow, _>(&mut listener)?;
        assert_eq!(listener.rows.len(), 1);
        Ok(())
    }

    #[test]
    fn trait_execute_runs_the_real_xlsx_parser() -> Result<()> {
        let file = write_xlsx();
        let mut options = ReadOptions::default();
        options.head_row_number = 1;
        let mut analyser =
            XlsxSaxAnalyser::from_path(file.path(), options.clone()).expect("analyser");
        let mut listener = CollectingListener::default();
        ExcelReadExecutor::execute::<DynamicRow, _>(&mut analyser, &options, &mut listener)?;
        assert_eq!(listener.rows.len(), 1);
        assert!(analyser.last_error().is_none());
        Ok(())
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use easyexcel_core::DynamicRow;
    use rust_xlsxwriter::Workbook;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_xlsx() -> NamedTempFile {
        let file = NamedTempFile::with_suffix(".xlsx").expect("temp xlsx");
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.write_string(0, 0, "name").expect("header");
        worksheet.write_string(1, 0, "alice").expect("row");
        workbook.save(file.path()).expect("save");
        file
    }

    #[test]
    fn new_rejects_workbooks_without_sheets() -> std::io::Result<()> {
        // 对应 Java：XlsxSaxAnalyser 构造时 sheetList 为空报错
        let directory = tempfile::tempdir()?;
        let path = directory.path().join("empty.xlsx");
        {
            let file = std::fs::File::create(&path)?;
            let mut writer = zip::ZipWriter::new(file);
            let options = zip::write::SimpleFileOptions::default();
            writer.start_file("[Content_Types].xml", options)?;
            writer.write_all(
                br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
</Types>"#,
            )?;
            writer.start_file("_rels/.rels", options)?;
            writer.write_all(
                br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
            )?;
            writer.start_file("xl/_rels/workbook.xml.rels", options)?;
            writer.write_all(
                br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#,
            )?;
            writer.start_file("xl/workbook.xml", options)?;
            writer.write_all(
                br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets></sheets></workbook>"#,
            )?;
            writer.finish()?;
        }
        let error = match XlsxSaxAnalyser::from_path(&path, ReadOptions::default()) {
            Ok(_) => panic!("empty workbook must fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("Can not find any sheet"));
        Ok(())
    }

    #[test]
    fn accessors_unsupported_entries_and_context() -> Result<()> {
        // 对应 Java：XlsxSaxAnalyser 公开访问器与 Unsupported 方法
        let file = write_xlsx();
        let options = ReadOptions::default();
        let analyser = XlsxSaxAnalyser::from_path(file.path(), options)?;

        assert_eq!(analyser.path(), file.path());
        assert!(
            analyser
                .xlsx_read_context()
                .analysis_context_impl()
                .analysis_context()
                .sheet_name()
                .is_empty()
        );
        assert!(analyser.analysis_context().sheet_name().is_empty());
        assert!(analyser.last_error().is_none());

        // readComments / parseXmlSource 由 read_xlsx 内部分派（对应 Java 方法签名保留）
        assert!(
            analyser
                .read_comments(&ReadSheet::default_construction())
                .is_err()
        );
        assert!(analyser.parse_xml_source().is_err());

        // 构造即可用（sheet_list 发现工作表）
        assert_eq!(analyser.sheet_list().len(), 1);
        Ok(())
    }

    #[test]
    fn execute_records_errors_in_last_error() -> Result<()> {
        // 对应 Java：execute 失败时持有异常供后续查询，成功后清除
        let file = write_xlsx();
        let mut options = ReadOptions::default();
        options.head_row_number = 1;
        let mut analyser = XlsxSaxAnalyser::from_path(file.path(), options)?;

        let mut failing = FailingListener;
        assert!(
            analyser
                .execute_with_listener::<DynamicRow, _>(&mut failing)
                .is_err()
        );
        assert!(analyser.last_error().is_some());

        let mut collecting = ExtraCollectingListener::default();
        analyser.execute_with_listener::<DynamicRow, _>(&mut collecting)?;
        assert_eq!(collecting.rows.len(), 1);
        assert!(analyser.last_error().is_none());
        Ok(())
    }

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
}
