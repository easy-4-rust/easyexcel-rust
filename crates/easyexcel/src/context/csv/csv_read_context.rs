//! 对应 Java：`com.alibaba.excel.context.csv.*`.

#[cfg(test)]
use crate::ReadOptions;
use crate::context::analysis_context::AnalysisContextLifecycle;
#[cfg(test)]
use crate::context::read_sheet::ReadSheet;
use crate::read::holder::csv::csv_read_sheet_holder::CsvReadSheetHolder;
use crate::read::holder::csv::csv_read_workbook_holder::CsvReadWorkbookHolder;
#[cfg(test)]
use crate::support::ExcelTypeEnum;
/// 对应 Java：`CsvReadContext extends AnalysisContext`.
pub trait CsvReadContext: AnalysisContextLifecycle {
    /// Returns CSV workbook holder. (Java `csvReadWorkbookHolder()`)
    fn csv_read_workbook_holder(&self) -> &CsvReadWorkbookHolder;

    /// Returns CSV sheet holder. (Java `csvReadSheetHolder()`)
    fn csv_read_sheet_holder(&self) -> Option<&CsvReadSheetHolder>;
}

pub use crate::context::csv::default_csv_read_context::DefaultCsvReadContext;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_sheet_materializes_typed_csv_holder() -> crate::core::Result<()> {
        // 对应 Java：CsvReadContext.currentSheet 物化 CsvReadSheetHolder
        let options = ReadOptions::default();
        let mut context = DefaultCsvReadContext::new(&options);
        assert!(context.csv_read_sheet_holder().is_none());
        assert_eq!(
            context.analysis_context_impl().excel_type(),
            ExcelTypeEnum::Csv
        );

        context.current_sheet(&ReadSheet::with_name(1, "Sheet1"))?;
        let holder = context.csv_read_sheet_holder().expect("csv sheet holder");
        assert_eq!(holder.inner().sheet_no, 1);
        assert_eq!(holder.inner().sheet_name, "Sheet1");
        assert_eq!(
            context
                .analysis_context_impl()
                .analysis_context()
                .sheet_name(),
            "Sheet1"
        );
        assert!(context.csv_read_workbook_holder().inner().ignore_empty_row);
        Ok(())
    }
}
