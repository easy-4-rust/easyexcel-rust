//! 对应 Java：`com.alibaba.excel.context.xlsx.*`.

#[cfg(test)]
use crate::ReadOptions;
use crate::context::analysis_context_impl::AnalysisContextImpl;
#[cfg(test)]
use crate::context::read_sheet::ReadSheet;
use crate::read::holder::xlsx::xlsx_read_sheet_holder::XlsxReadSheetHolder;
use crate::read::holder::xlsx::xlsx_read_workbook_holder::XlsxReadWorkbookHolder;
#[cfg(test)]
use crate::support::ExcelTypeEnum;
/// 对应 Java：`XlsxReadContext extends AnalysisContext`.
pub trait XlsxReadContext {
    /// Returns the shared analysis state. (Java `AnalysisContext` methods)
    fn analysis_context_impl(&self) -> &AnalysisContextImpl;

    /// Returns XLSX workbook holder. (Java `xlsxReadWorkbookHolder()`)
    fn xlsx_read_workbook_holder(&self) -> &XlsxReadWorkbookHolder;

    /// Returns XLSX sheet holder. (Java `xlsxReadSheetHolder()`)
    fn xlsx_read_sheet_holder(&self) -> Option<&XlsxReadSheetHolder>;
}

pub use crate::context::xlsx::default_xlsx_read_context::DefaultXlsxReadContext;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sheet_materializes_typed_xlsx_holder() -> crate::core::Result<()> {
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
