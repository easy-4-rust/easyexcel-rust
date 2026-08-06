//! 对应 Java：`com.alibaba.excel.analysis.ExcelReadExecutor` (interface).

use std::path::PathBuf;

use crate::core::{ExcelRow, ReadListener, Result};
use crate::support::ExcelTypeEnum;

use crate::ReadOptions;
use crate::analysis::csv::csv_excel_read_executor::CsvExcelReadExecutor;
use crate::analysis::v03::xls_sax_analyser::XlsSaxAnalyser;
use crate::analysis::v07::xlsx_sax_analyser::XlsxSaxAnalyser;
use crate::context::ReadSheet;

/// 对应 Java：`ExcelReadExecutor`.
///
/// Java declares `sheetList()` and `execute()`. Rust's `read_xlsx` /
/// `read_xls` / `read_csv` functions cover the same contract.
pub trait ExcelReadExecutor {
    /// Returns discovered worksheets. (Java `sheetList()`)
    fn sheet_list(&self) -> &[ReadSheet];

    /// Executes the read with Rust's typed listener and current options.
    ///
    /// Java retrieves erased listeners and sheet parameters from
    /// `ReadWorkbook`; Rust makes those dependencies explicit.
    ///
    /// # Errors
    ///
    /// 当工作簿解析（SAX/记录读取）失败时返回 `ExcelError`。
    fn execute<T, L>(&mut self, options: &ReadOptions, listener: &mut L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>;
}

include!("excel_read_executor/excel_read_executor_kind.rs");

#[cfg(test)]
mod tests_extra {
    use super::*;
    use crate::core::DynamicRow;
    use rust_xlsxwriter::Workbook;
    use tempfile::NamedTempFile;

    #[derive(Default)]
    struct CollectingListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for CollectingListener {
        fn invoke(
            &mut self,
            data: DynamicRow,
            _context: &crate::core::AnalysisContext,
        ) -> Result<()> {
            self.rows.push(data);
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
    fn xlsx_executor_variant_reads_and_reports_type() -> Result<()> {
        // 对应 Java：choiceExcelExecutor 选择 XlsxSaxAnalyser 后执行
        let file = write_xlsx();
        let options = ReadOptions {
            head_row_number: 1,
            ..ReadOptions::default()
        };
        let mut executor =
            ExcelReadExecutorKind::new(ExcelTypeEnum::Xlsx, file.path(), options.clone())?;
        assert_eq!(executor.excel_type(), ExcelTypeEnum::Xlsx);
        assert_eq!(executor.sheet_list().len(), 1);
        assert!(!executor.sheet_list()[0].sheet_name().is_empty());

        let mut listener = CollectingListener::default();
        executor.execute_with_listener(&options, &mut listener)?;
        assert_eq!(listener.rows.len(), 1);
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
    use tempfile::NamedTempFile;

    use super::*;

    #[derive(Default)]
    struct XlsCollectingListener {
        rows: Vec<DynamicRow>,
    }

    impl ReadListener<DynamicRow> for XlsCollectingListener {
        fn invoke(
            &mut self,
            data: DynamicRow,
            _context: &crate::core::AnalysisContext,
        ) -> Result<()> {
            self.rows.push(data);
            Ok(())
        }
    }

    /// 物化 Java 官方多 sheet .xls fixture（与 `xls_sax_analyser` 测试共用）。
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
    fn xls_executor_variant_discovers_sheets_and_reads_rows() -> Result<()> {
        // 对应 Java：choiceExcelExecutor 选择 XlsSaxAnalyser 后执行
        let file = write_java_multisheet_xls();
        let options = ReadOptions {
            head_row_number: 1,
            sheet: crate::SheetSelector::Index(0),
            ..ReadOptions::default()
        };
        let mut executor =
            ExcelReadExecutorKind::new(ExcelTypeEnum::Xls, file.path(), options.clone())?;
        assert_eq!(executor.excel_type(), ExcelTypeEnum::Xls);
        assert!(!executor.sheet_list().is_empty());

        let mut listener = XlsCollectingListener::default();
        executor.execute_with_listener(&options, &mut listener)?;
        assert!(!listener.rows.is_empty());
        Ok(())
    }

    #[test]
    fn csv_executor_variant_reports_its_type() -> Result<()> {
        // 对应 Java：CSV executor 的 excelType 与 sheetList 契约
        let file = NamedTempFile::with_suffix(".csv").expect("temp csv");
        fs::write(file.path(), "a,b\n1,2\n").expect("write csv");
        let executor =
            ExcelReadExecutorKind::new(ExcelTypeEnum::Csv, file.path(), ReadOptions::default())?;
        assert_eq!(executor.excel_type(), ExcelTypeEnum::Csv);
        assert_eq!(executor.sheet_list().len(), 1);
        Ok(())
    }

    #[test]
    fn executor_creation_rejects_unsupported_workbook_paths() {
        // 对应 Java：损坏/缺失的 xlsx 文件在选择 executor 时报错
        let directory = tempfile::tempdir().expect("tempdir");
        let missing = directory.path().join("missing.xlsx");
        assert!(
            ExcelReadExecutorKind::new(ExcelTypeEnum::Xlsx, &missing, ReadOptions::default(),)
                .is_err()
        );
    }
}
