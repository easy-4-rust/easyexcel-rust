//! 对应 Java：`com.alibaba.excel.context.csv.DefaultCsvReadContext`

use crate::ReadOptions;
use crate::context::analysis_context_impl::AnalysisContextImpl;
use crate::context::csv::csv_read_context::CsvReadContext;
use crate::context::read_sheet::ReadSheet;
use crate::read::holder::csv::csv_read_sheet_holder::CsvReadSheetHolder;
use crate::read::holder::csv::csv_read_workbook_holder::CsvReadWorkbookHolder;
use crate::support::ExcelTypeEnum;

/// 对应 Java：`DefaultCsvReadContext extends AnalysisContextImpl implements CsvReadContext`.
#[derive(Debug, Clone)]
pub struct DefaultCsvReadContext {
    inner: AnalysisContextImpl,
    csv_read_workbook_holder: CsvReadWorkbookHolder,
    csv_read_sheet_holder: Option<CsvReadSheetHolder>,
}
impl DefaultCsvReadContext {
    /// 对应 Java：`DefaultCsvReadContext(ReadWorkbook, ExcelTypeEnum)`.
    #[must_use]
    pub fn new(options: &ReadOptions) -> Self {
        Self {
            inner: AnalysisContextImpl::new(ExcelTypeEnum::Csv, options),
            csv_read_workbook_holder: CsvReadWorkbookHolder::from_options(options),
            csv_read_sheet_holder: None,
        }
    }

    /// Selects the current sheet and materializes the typed CSV holder.
    ///
    /// # Errors
    ///
    /// 当 `read_sheet.sheet_no()` 超出 `i32` 范围时返回 [`ExcelError::Format`]。
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
