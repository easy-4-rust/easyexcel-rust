//! 对应 Java：`com.alibaba.excel.context.xlsx.DefaultXlsxReadContext`

use crate::ReadOptions;
use crate::context::analysis_context_impl::AnalysisContextImpl;
use crate::context::read_sheet::ReadSheet;
use crate::context::xlsx::xlsx_read_context::XlsxReadContext;
use crate::read::holder::xlsx::xlsx_read_sheet_holder::XlsxReadSheetHolder;
use crate::read::holder::xlsx::xlsx_read_workbook_holder::XlsxReadWorkbookHolder;
use crate::read::metadata::ReadWorkbook;
use crate::support::ExcelTypeEnum;

/// 对应 Java：`DefaultXlsxReadContext extends AnalysisContextImpl implements XlsxReadContext`.
#[derive(Debug, Clone)]
pub struct DefaultXlsxReadContext {
    /// Shared analysis state.
    inner: AnalysisContextImpl,
    /// XLSX workbook holder.
    xlsx_read_workbook_holder: XlsxReadWorkbookHolder,
    /// Active XLSX sheet holder.
    xlsx_read_sheet_holder: Option<XlsxReadSheetHolder>,
}
impl DefaultXlsxReadContext {
    /// 对应 Java：`DefaultXlsxReadContext(ReadWorkbook, ExcelTypeEnum)`.
    #[must_use]
    pub fn new(options: &ReadOptions) -> Self {
        Self {
            inner: AnalysisContextImpl::new(ExcelTypeEnum::Xlsx, options),
            xlsx_read_workbook_holder: XlsxReadWorkbookHolder::from_options(options),
            xlsx_read_sheet_holder: None,
        }
    }

    /// 使用 Java `ReadWorkbook` 与实际格式创建 XLSX 读取上下文。
    ///
    /// 对应 Java：`DefaultXlsxReadContext(ReadWorkbook, ExcelTypeEnum)`。
    #[must_use]
    pub fn from_read_workbook(
        read_workbook: &ReadWorkbook,
        actual_excel_type: ExcelTypeEnum,
    ) -> Self {
        let options = read_workbook.options();
        Self {
            inner: AnalysisContextImpl::new(actual_excel_type, options),
            xlsx_read_workbook_holder: XlsxReadWorkbookHolder::from_options(options),
            xlsx_read_sheet_holder: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.context.xlsx.DefaultXlsxReadContext。 Selects the current sheet and materializes the typed XLSX holder.
    ///
    /// # Errors
    ///
    /// 当 `read_sheet.sheet_no()` 超出 `i32` 范围时返回 `ExcelError::Format`。
    pub fn current_sheet(&mut self, read_sheet: &ReadSheet) -> crate::core::Result<()> {
        self.inner.current_sheet(read_sheet)?;
        let sheet_no = i32::try_from(read_sheet.sheet_no()).map_err(|_| {
            crate::core::ExcelError::Format("sheet index exceeds i32 range".to_owned())
        })?;
        self.xlsx_read_sheet_holder =
            Some(XlsxReadSheetHolder::new(sheet_no, read_sheet.sheet_name()));
        Ok(())
    }
}
impl XlsxReadContext for DefaultXlsxReadContext {
    fn analysis_context_impl(&self) -> &AnalysisContextImpl {
        &self.inner
    }

    fn xlsx_read_workbook_holder(&self) -> &XlsxReadWorkbookHolder {
        &self.xlsx_read_workbook_holder
    }

    fn xlsx_read_sheet_holder(&self) -> Option<&XlsxReadSheetHolder> {
        self.xlsx_read_sheet_holder.as_ref()
    }
}
