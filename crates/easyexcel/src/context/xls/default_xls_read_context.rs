//! 对应 Java：`com.alibaba.excel.context.xls.DefaultXlsReadContext`

use crate::ReadOptions;
use crate::context::analysis_context_impl::AnalysisContextImpl;
use crate::context::read_sheet::ReadSheet;
use crate::context::xls::xls_read_context::XlsReadContext;
use crate::read::holder::xls::xls_read_sheet_holder::XlsReadSheetHolder;
use crate::read::holder::xls::xls_read_workbook_holder::XlsReadWorkbookHolder;
use crate::support::ExcelTypeEnum;

/// 对应 Java：`DefaultXlsReadContext extends AnalysisContextImpl implements XlsReadContext`.
#[derive(Debug, Clone)]
pub struct DefaultXlsReadContext {
    inner: AnalysisContextImpl,
    xls_read_workbook_holder: XlsReadWorkbookHolder,
    xls_read_sheet_holder: Option<XlsReadSheetHolder>,
}
impl DefaultXlsReadContext {
    /// 对应 Java：`DefaultXlsReadContext(ReadWorkbook, ExcelTypeEnum)`.
    #[must_use]
    pub fn new(options: &ReadOptions) -> Self {
        Self {
            inner: AnalysisContextImpl::new(ExcelTypeEnum::Xls, options),
            xls_read_workbook_holder: XlsReadWorkbookHolder::from_options(options),
            xls_read_sheet_holder: None,
        }
    }

    /// Selects the current sheet and materializes the typed XLS holder.
    ///
    /// # Errors
    ///
    /// 当 `read_sheet.sheet_no()` 超出 `i32` 范围时返回 [`ExcelError::Format`]。
    pub fn current_sheet(&mut self, read_sheet: &ReadSheet) -> crate::core::Result<()> {
        self.inner.current_sheet(read_sheet)?;
        let sheet_no = i32::try_from(read_sheet.sheet_no()).map_err(|_| {
            crate::core::ExcelError::Format("sheet index exceeds i32 range".to_owned())
        })?;
        self.xls_read_sheet_holder =
            Some(XlsReadSheetHolder::new(sheet_no, read_sheet.sheet_name()));
        Ok(())
    }

    /// Returns mutable XLS workbook state for record listeners.
    pub const fn xls_read_workbook_holder_mut(&mut self) -> &mut XlsReadWorkbookHolder {
        &mut self.xls_read_workbook_holder
    }
}
impl XlsReadContext for DefaultXlsReadContext {
    fn analysis_context_impl(&self) -> &AnalysisContextImpl {
        &self.inner
    }

    fn xls_read_workbook_holder(&self) -> &XlsReadWorkbookHolder {
        &self.xls_read_workbook_holder
    }

    fn xls_read_sheet_holder(&self) -> Option<&XlsReadSheetHolder> {
        self.xls_read_sheet_holder.as_ref()
    }
}
