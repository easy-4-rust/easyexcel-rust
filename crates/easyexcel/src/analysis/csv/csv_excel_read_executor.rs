//! 对应 Java：`com.alibaba.excel.analysis.csv.CsvExcelReadExecutor`.

use std::path::PathBuf;

use crate::core::{ExcelError, ExcelRow, ReadListener, Result};

use crate::analysis::excel_read_executor::{ExcelReadExecutor, NoopDynamicReadListener};
#[cfg(test)]
use crate::context::CsvReadContext;
use crate::context::{DefaultCsvReadContext, ReadSheet};
use crate::read::metadata::ReadWorkbook;
use crate::support::ExcelTypeEnum;
use crate::{ReadOptions, read_csv};

/// 对应 Java：`CsvExcelReadExecutor implements ExcelReadExecutor`.
///
/// The actual CSV parsing in Rust lives in `crate::read_csv`. This
/// struct exists for 1:1 Java package parity.
#[derive(Debug, Clone)]
pub struct CsvExcelReadExecutor {
    /// Single logical sheet. (Java `sheetList`)
    sheet_list: Vec<ReadSheet>,
    /// CSV input path supplied by `ExcelAnalyserImpl`.
    path: Option<PathBuf>,
    /// 由分析器解析后的工作簿读取选项。
    options: ReadOptions,
    /// Java 构造器传入并由执行器持有的 CSV 读取上下文。
    csv_read_context: DefaultCsvReadContext,
}

impl CsvExcelReadExecutor {
    /// 使用 CSV 读取上下文创建执行器。
    ///
    /// 对应 Java：`CsvExcelReadExecutor(CsvReadContext)`。
    #[must_use]
    pub fn new(csv_read_context: DefaultCsvReadContext) -> Self {
        let path = csv_read_context.file().map(PathBuf::from);
        let options = csv_read_context.options().clone();
        Self {
            sheet_list: vec![ReadSheet::new(0)],
            path,
            options,
            csv_read_context,
        }
    }

    /// 对应 Java：com.alibaba.excel.analysis.csv.CsvExcelReadExecutor。 Creates an executor bound to a real CSV input.
    #[must_use]
    pub fn from_path(path: impl Into<PathBuf>) -> Self {
        Self::from_path_with_options(path, ReadOptions::default())
    }

    /// 创建绑定真实 CSV 输入及解析选项的执行器。
    #[must_use]
    pub fn from_path_with_options(path: impl Into<PathBuf>, options: ReadOptions) -> Self {
        let mut read_workbook = ReadWorkbook::from(options);
        read_workbook.set_file(path);
        Self::new(DefaultCsvReadContext::from_read_workbook(
            &read_workbook,
            ExcelTypeEnum::Csv,
        ))
    }

    /// 返回构造器传入并由执行器持有的 CSV 上下文。
    #[must_use]
    pub const fn csv_read_context(&self) -> &DefaultCsvReadContext {
        &self.csv_read_context
    }
}

impl ExcelReadExecutor for CsvExcelReadExecutor {
    /// 对应 Java：`sheetList()`.
    fn sheet_list(&self) -> &[ReadSheet] {
        &self.sheet_list
    }

    /// 对应 Java：`execute()` through the real CSV record parser.
    fn execute(&mut self) -> Result<()> {
        let options = self.options.clone();
        self.csv_read_context.current_sheet(&self.sheet_list[0])?;
        let result = self.execute_with_listener::<crate::core::DynamicRow, _>(
            &options,
            &mut NoopDynamicReadListener,
        );
        if result.is_ok() {
            self.csv_read_context.mark_parser_initialized();
        }
        result
    }

    fn execute_with_listener<T, L>(&mut self, options: &ReadOptions, listener: &mut L) -> Result<()>
    where
        T: ExcelRow,
        L: ReadListener<T>,
    {
        let path = self.path.as_deref().ok_or_else(|| {
            ExcelError::Format("CsvExcelReadExecutor requires an input path".to_owned())
        })?;
        read_csv::<T, L>(path, options, listener)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::core::{AnalysisContext, DynamicRow};
    use tempfile::NamedTempFile;

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
    }

    #[test]
    fn trait_execute_runs_the_real_csv_parser() -> Result<()> {
        let mut file = NamedTempFile::with_suffix(".csv")?;
        writeln!(file, "value")?;
        writeln!(file, "csv-row")?;
        let options = ReadOptions {
            head_row_number: 1,
            ..ReadOptions::default()
        };
        let mut executor =
            CsvExcelReadExecutor::from_path_with_options(file.path(), options.clone());
        ExcelReadExecutor::execute(&mut executor)?;

        let mut listener = CollectingListener::default();

        ExcelReadExecutor::execute_with_listener::<DynamicRow, _>(
            &mut executor,
            &options,
            &mut listener,
        )?;

        assert_eq!(listener.rows.len(), 1);
        assert_eq!(executor.sheet_list()[0].sheet_name(), "");
        assert!(
            executor
                .csv_read_context()
                .csv_read_sheet_holder()
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn unbound_executor_reports_a_real_configuration_error() {
        let mut executor =
            CsvExcelReadExecutor::new(DefaultCsvReadContext::new(&ReadOptions::default()));
        let mut listener = CollectingListener::default();
        let error = ExcelReadExecutor::execute_with_listener::<DynamicRow, _>(
            &mut executor,
            &ReadOptions::default(),
            &mut listener,
        )
        .expect_err("unbound CSV executor must fail");
        assert!(error.to_string().contains("requires an input path"));
    }
}
