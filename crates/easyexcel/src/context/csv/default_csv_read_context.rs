//! 对应 Java：`com.alibaba.excel.context.csv.DefaultCsvReadContext`

use std::path::{Path, PathBuf};

use crate::ReadOptions;
use crate::context::analysis_context_impl::AnalysisContextImpl;
use crate::context::csv::csv_read_context::CsvReadContext;
use crate::context::read_sheet::ReadSheet;
use crate::read::holder::csv::csv_read_sheet_holder::CsvReadSheetHolder;
use crate::read::holder::csv::csv_read_workbook_holder::CsvReadWorkbookHolder;
use crate::read::metadata::ReadWorkbook;
use crate::support::ExcelTypeEnum;

/// 对应 Java：`DefaultCsvReadContext extends AnalysisContextImpl implements CsvReadContext`.
#[derive(Debug, Clone)]
pub struct DefaultCsvReadContext {
    inner: AnalysisContextImpl,
    csv_read_workbook_holder: CsvReadWorkbookHolder,
    csv_read_sheet_holder: Option<CsvReadSheetHolder>,
    file: Option<PathBuf>,
    options: ReadOptions,
    parser_initialized: bool,
}
impl DefaultCsvReadContext {
    /// 对应 Java：`DefaultCsvReadContext(ReadWorkbook, ExcelTypeEnum)`.
    #[must_use]
    pub fn new(options: &ReadOptions) -> Self {
        Self {
            inner: AnalysisContextImpl::new(ExcelTypeEnum::Csv, options),
            csv_read_workbook_holder: CsvReadWorkbookHolder::from_options(options),
            csv_read_sheet_holder: None,
            file: None,
            options: options.clone(),
            parser_initialized: false,
        }
    }

    /// 使用 Java `ReadWorkbook` 与实际格式创建 CSV 读取上下文。
    ///
    /// 对应 Java：`DefaultCsvReadContext(ReadWorkbook, ExcelTypeEnum)`。
    #[must_use]
    pub fn from_read_workbook(
        read_workbook: &ReadWorkbook,
        actual_excel_type: ExcelTypeEnum,
    ) -> Self {
        let options = read_workbook.options().clone();
        let file = read_workbook.file().map(Path::to_path_buf);
        Self {
            inner: AnalysisContextImpl::new(actual_excel_type, &options),
            csv_read_workbook_holder: CsvReadWorkbookHolder::from_options(&options),
            csv_read_sheet_holder: None,
            file,
            options,
            parser_initialized: false,
        }
    }

    /// 返回上下文持有的 CSV 输入文件。
    #[must_use]
    pub fn file(&self) -> Option<&Path> {
        self.file.as_deref()
    }

    /// 返回上下文持有的有效读取选项。
    #[must_use]
    pub const fn options(&self) -> &ReadOptions {
        &self.options
    }

    /// 返回执行器是否已经初始化 CSV 解析器。
    #[must_use]
    pub const fn parser_initialized(&self) -> bool {
        self.parser_initialized
    }

    /// 记录 CSV 解析器已初始化；由 `CsvExcelReadExecutor` 在执行入口调用。
    pub(crate) fn mark_parser_initialized(&mut self) {
        self.parser_initialized = true;
    }

    /// 对应 Java：com.alibaba.excel.context.csv.DefaultCsvReadContext。 Selects the current sheet and materializes the typed CSV holder.
    ///
    /// # Errors
    ///
    /// 当 `read_sheet.sheet_no()` 超出 `i32` 范围时返回 `ExcelError::Format`。
    pub fn current_sheet(&mut self, read_sheet: &ReadSheet) -> crate::core::Result<()> {
        self.inner.current_sheet(read_sheet)?;
        let sheet_no = i32::try_from(read_sheet.sheet_no()).map_err(|_| {
            crate::core::ExcelError::Format("sheet index exceeds i32 range".to_owned())
        })?;
        self.csv_read_sheet_holder =
            Some(CsvReadSheetHolder::new(sheet_no, read_sheet.sheet_name()));
        Ok(())
    }
}
impl CsvReadContext for DefaultCsvReadContext {
    fn analysis_context_impl(&self) -> &AnalysisContextImpl {
        &self.inner
    }

    fn csv_read_workbook_holder(&self) -> &CsvReadWorkbookHolder {
        &self.csv_read_workbook_holder
    }

    fn csv_read_sheet_holder(&self) -> Option<&CsvReadSheetHolder> {
        self.csv_read_sheet_holder.as_ref()
    }
}
