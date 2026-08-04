//! 对应 Java：`com.alibaba.excel.context.xls.*`.

use crate::support::ExcelTypeEnum;

use crate::ReadOptions;
use crate::context::read_sheet::ReadSheet;
use crate::read::holder::xls::xls_read_sheet_holder::XlsReadSheetHolder;
use crate::read::holder::xls::xls_read_workbook_holder::XlsReadWorkbookHolder;

use crate::context::analysis_context_impl::AnalysisContextImpl;

/// 对应 Java：`XlsReadContext extends AnalysisContext`.
pub trait XlsReadContext {
    /// Returns the shared analysis state.
    fn analysis_context_impl(&self) -> &AnalysisContextImpl;

    /// Returns XLS workbook holder. (Java `xlsReadWorkbookHolder()`)
    fn xls_read_workbook_holder(&self) -> &XlsReadWorkbookHolder;

    /// Returns XLS sheet holder. (Java `xlsReadSheetHolder()`)
    fn xls_read_sheet_holder(&self) -> Option<&XlsReadSheetHolder>;
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sheet_materializes_typed_xls_holder() -> crate::core::Result<()> {
        // 对应 Java：XlsReadContext.currentSheet 物化 XlsReadSheetHolder
        let options = ReadOptions::default();
        let mut context = DefaultXlsReadContext::new(&options);
        assert!(context.xls_read_sheet_holder().is_none());
        assert_eq!(
            context.analysis_context_impl().excel_type(),
            ExcelTypeEnum::Xls
        );

        context.current_sheet(&ReadSheet::with_name(0, "Sheet1"))?;
        let holder = context.xls_read_sheet_holder().expect("xls sheet holder");
        assert_eq!(holder.inner().sheet_no, 0);
        assert_eq!(holder.inner().sheet_name, "Sheet1");
        assert_eq!(
            context
                .analysis_context_impl()
                .analysis_context()
                .sheet_name(),
            "Sheet1"
        );
        assert!(context.xls_read_workbook_holder().inner().ignore_empty_row);
        Ok(())
    }
}
