//! 对应 Java：`com.alibaba.excel.context.csv.*`.

use crate::support::ExcelTypeEnum;

use crate::ReadOptions;
use crate::context::read_sheet::ReadSheet;
use crate::read::holder::csv::csv_read_sheet_holder::CsvReadSheetHolder;
use crate::read::holder::csv::csv_read_workbook_holder::CsvReadWorkbookHolder;

use super::analysis_context_impl::AnalysisContextImpl;

/// 对应 Java：`CsvReadContext extends AnalysisContext`.
pub trait CsvReadContext {
    /// Returns the shared analysis state.
    fn analysis_context_impl(&self) -> &AnalysisContextImpl;

    /// Returns CSV workbook holder. (Java `csvReadWorkbookHolder()`)
    fn csv_read_workbook_holder(&self) -> &CsvReadWorkbookHolder;

    /// Returns CSV sheet holder. (Java `csvReadSheetHolder()`)
    fn csv_read_sheet_holder(&self) -> Option<&CsvReadSheetHolder>;
}

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
