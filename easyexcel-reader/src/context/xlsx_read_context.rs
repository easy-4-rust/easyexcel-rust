//! 对应 Java：`com.alibaba.excel.context.xlsx.*`.

use easyexcel_core::support::ExcelTypeEnum;

use crate::ReadOptions;
use crate::context::read_sheet::ReadSheet;
use crate::holder::xlsx::xlsx_read_sheet_holder::XlsxReadSheetHolder;
use crate::holder::xlsx::xlsx_read_workbook_holder::XlsxReadWorkbookHolder;

use super::analysis_context_impl::AnalysisContextImpl;

/// 对应 Java：`XlsxReadContext extends AnalysisContext`.
pub trait XlsxReadContext {
    /// Returns the shared analysis state. (Java `AnalysisContext` methods)
    fn analysis_context_impl(&self) -> &AnalysisContextImpl;

    /// Returns XLSX workbook holder. (Java `xlsxReadWorkbookHolder()`)
    fn xlsx_read_workbook_holder(&self) -> &XlsxReadWorkbookHolder;

    /// Returns XLSX sheet holder. (Java `xlsxReadSheetHolder()`)
    fn xlsx_read_sheet_holder(&self) -> Option<&XlsxReadSheetHolder>;
}

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

    /// Selects the current sheet and materializes the typed XLSX holder.
    ///
    /// # Errors
    ///
    /// 当 `read_sheet.sheet_no()` 超出 `i32` 范围时返回 [`ExcelError::Format`]。
    pub fn current_sheet(&mut self, read_sheet: &ReadSheet) -> easyexcel_core::Result<()> {
        self.inner.current_sheet(read_sheet)?;
        let sheet_no = i32::try_from(read_sheet.sheet_no()).map_err(|_| {
            easyexcel_core::ExcelError::Format("sheet index exceeds i32 range".to_owned())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sheet_materializes_typed_xlsx_holder() -> easyexcel_core::Result<()> {
        // 对应 Java：XlsxReadContext.currentSheet 物化 XlsxReadSheetHolder
        let options = ReadOptions::default();
        let mut context = DefaultXlsxReadContext::new(&options);
        assert!(context.xlsx_read_sheet_holder().is_none());
        assert_eq!(
            context.analysis_context_impl().excel_type(),
            ExcelTypeEnum::Xlsx
        );

        context.current_sheet(&ReadSheet::with_name(2, "Sheet2"))?;
        let holder = context.xlsx_read_sheet_holder().expect("xlsx sheet holder");
        assert_eq!(holder.inner().sheet_no, 2);
        assert_eq!(holder.inner().sheet_name, "Sheet2");
        assert_eq!(
            context
                .analysis_context_impl()
                .analysis_context()
                .sheet_name(),
            "Sheet2"
        );
        assert!(context.xlsx_read_workbook_holder().inner().ignore_empty_row);
        Ok(())
    }
}
