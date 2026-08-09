//! 对应 Java：`com.alibaba.excel.context.xls.*`.

#[cfg(test)]
use crate::ReadOptions;
use crate::context::analysis_context::AnalysisContextLifecycle;
#[cfg(test)]
use crate::context::read_sheet::ReadSheet;
use crate::read::holder::xls::xls_read_sheet_holder::XlsReadSheetHolder;
use crate::read::holder::xls::xls_read_workbook_holder::XlsReadWorkbookHolder;
#[cfg(test)]
use crate::support::ExcelTypeEnum;
/// 对应 Java：`XlsReadContext extends AnalysisContext`.
pub trait XlsReadContext: AnalysisContextLifecycle {
    /// Returns XLS workbook holder. (Java `xlsReadWorkbookHolder()`)
    fn xls_read_workbook_holder(&self) -> &XlsReadWorkbookHolder;

    /// Returns XLS sheet holder. (Java `xlsReadSheetHolder()`)
    fn xls_read_sheet_holder(&self) -> Option<&XlsReadSheetHolder>;
}

pub use crate::context::xls::default_xls_read_context::DefaultXlsReadContext;

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
