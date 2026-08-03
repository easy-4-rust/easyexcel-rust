//! 对应 Java：`com.alibaba.excel.context.AnalysisContextImpl`.

use std::collections::HashSet;

use easyexcel_core::support::ExcelTypeEnum;
use easyexcel_core::{AnalysisContext, CustomReadObject, ExcelError, Result};

use crate::ReadOptions;
use crate::holder::read_row_holder::ReadRowHolder;
use crate::holder::read_sheet_holder::ReadSheetHolder;
use crate::holder::read_workbook_holder::ReadWorkbookHolder;
use crate::processor::analysis_event_processor::AnalysisEventProcessor;
use crate::processor::default_analysis_event_processor::DefaultAnalysisEventProcessor;

use super::read_sheet::ReadSheet;

/// 对应 Java：`AnalysisContextImpl implements AnalysisContext`.
///
/// Wraps the listener-facing [`AnalysisContext`] from `easyexcel-core` and
/// attaches holder state that Java stores on this type.
#[derive(Debug, Clone)]
pub struct AnalysisContextImpl {
    /// Listener callback context. (Java row/sheet fields on holders)
    inner: AnalysisContext,
    /// Resolved workbook format. (Java `readWorkbookHolder.getExcelType()`)
    excel_type: ExcelTypeEnum,
    /// Workbook holder. (Java `readWorkbookHolder`)
    read_workbook_holder: ReadWorkbookHolder,
    /// Active sheet holder. (Java `readSheetHolder`)
    read_sheet_holder: Option<ReadSheetHolder>,
    /// Active row holder. (Java `readRowHolder`)
    read_row_holder: Option<ReadRowHolder>,
    /// Sheets requested by the caller. (Java `readSheetList`)
    read_sheet_list: Option<Vec<ReadSheet>>,
    /// Event processor. (Java `analysisEventProcessor`)
    analysis_event_processor: DefaultAnalysisEventProcessor,
    /// Prevents duplicate sheet reads. (Java `hasReadSheet`)
    has_read_sheet: HashSet<i32>,
}

impl AnalysisContextImpl {
    /// 对应 Java：`AnalysisContextImpl(ReadWorkbook, ExcelTypeEnum)`.
    #[must_use]
    pub fn new(excel_type: ExcelTypeEnum, options: &ReadOptions) -> Self {
        Self {
            inner: AnalysisContext::new("", 0, 0).with_custom_object(options.custom_object.clone()),
            excel_type,
            read_workbook_holder: ReadWorkbookHolder::from_options(options),
            read_sheet_holder: None,
            read_row_holder: None,
            read_sheet_list: None,
            analysis_event_processor: DefaultAnalysisEventProcessor,
            has_read_sheet: HashSet::new(),
        }
    }

    /// Returns the listener callback context. (Java deprecated getters collapse here)
    #[must_use]
    pub fn analysis_context(&self) -> &AnalysisContext {
        &self.inner
    }

    /// Returns a mutable listener callback context.
    #[must_use]
    pub fn analysis_context_mut(&mut self) -> &mut AnalysisContext {
        &mut self.inner
    }

    /// 对应 Java：`currentSheet(ReadSheet)`.
    ///
    /// # Errors
    ///
    /// Returns when the same sheet is read twice, matching Java
    /// `ExcelAnalysisException("Cannot read sheet repeatedly.")`.
    pub fn current_sheet(&mut self, read_sheet: &ReadSheet) -> Result<()> {
        let sheet_no = i32::try_from(read_sheet.sheet_no())
            .map_err(|_| ExcelError::Format("sheet index exceeds i32 range".to_owned()))?;
        if self.has_read_sheet.contains(&sheet_no) {
            return Err(ExcelError::Format(
                "Cannot read sheet repeatedly.".to_owned(),
            ));
        }
        self.has_read_sheet.insert(sheet_no);
        self.read_sheet_holder = Some(ReadSheetHolder::new(sheet_no, read_sheet.sheet_name()));
        self.inner = AnalysisContext::new(
            read_sheet.sheet_name(),
            read_sheet.sheet_no(),
            self.inner.row_index(),
        )
        .with_custom_object(self.inner.custom_object().cloned());
        Ok(())
    }

    /// Returns the workbook holder. (Java `readWorkbookHolder()`)
    #[must_use]
    pub const fn read_workbook_holder(&self) -> &ReadWorkbookHolder {
        &self.read_workbook_holder
    }

    /// Returns the active sheet holder. (Java `readSheetHolder()`)
    #[must_use]
    pub const fn read_sheet_holder(&self) -> Option<&ReadSheetHolder> {
        self.read_sheet_holder.as_ref()
    }

    /// Returns the active row holder. (Java `readRowHolder()`)
    #[must_use]
    pub const fn read_row_holder(&self) -> Option<&ReadRowHolder> {
        self.read_row_holder.as_ref()
    }

    /// Sets the active row holder. (Java `readRowHolder(ReadRowHolder)`)
    pub fn set_read_row_holder(&mut self, read_row_holder: ReadRowHolder) {
        self.read_row_holder = Some(read_row_holder);
    }

    /// Returns the custom read object. (Java `getCustom()`)
    #[must_use]
    pub fn custom(&self) -> Option<&CustomReadObject> {
        self.inner.custom_object()
    }

    /// Returns the event processor. (Java `analysisEventProcessor()`)
    pub fn analysis_event_processor(&mut self) -> &mut dyn AnalysisEventProcessor {
        &mut self.analysis_event_processor
    }

    /// Returns requested sheets. (Java `readSheetList()`)
    #[must_use]
    pub fn read_sheet_list(&self) -> Option<&[ReadSheet]> {
        self.read_sheet_list.as_deref()
    }

    /// Sets requested sheets. (Java `readSheetList(List<ReadSheet>)`)
    pub fn set_read_sheet_list(&mut self, read_sheet_list: Vec<ReadSheet>) {
        self.read_sheet_list = Some(read_sheet_list);
    }

    /// Returns the resolved workbook format. (Java `@Deprecated getExcelType()`)
    #[must_use]
    pub const fn excel_type(&self) -> ExcelTypeEnum {
        self.excel_type
    }

    /// 对应 Java：`@Deprecated getCurrentRowNum()`.
    #[must_use]
    pub fn current_row_num(&self) -> Option<i32> {
        self.read_row_holder.as_ref().map(|holder| holder.row_index)
    }

    /// 对应 Java：`@Deprecated interrupt()`.
    ///
    /// # Errors
    ///
    /// Always returns `ExcelError::Unsupported` — Rust listeners use
    /// [`easyexcel_core::ReadListener::has_next`] instead.
    pub fn interrupt(&self) -> Result<()> {
        Err(ExcelError::Unsupported(
            "AnalysisContextImpl.interrupt is deprecated; use ReadListener::has_next".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use easyexcel_core::support::ExcelTypeEnum;

    #[test]
    fn current_sheet_updates_listener_context() -> Result<()> {
        let options = ReadOptions::default();
        let mut context = AnalysisContextImpl::new(ExcelTypeEnum::Xlsx, &options);
        context.current_sheet(&ReadSheet::with_name(0, "Sheet1"))?;
        assert_eq!(context.analysis_context().sheet_name(), "Sheet1");
        assert_eq!(context.analysis_context().sheet_no(), 0);
        Ok(())
    }

    #[test]
    fn repeated_sheet_read_matches_java_error() {
        let options = ReadOptions::default();
        let mut context = AnalysisContextImpl::new(ExcelTypeEnum::Xlsx, &options);
        let sheet = ReadSheet::with_name(0, "Sheet1");
        context.current_sheet(&sheet).expect("first read");
        let error = context.current_sheet(&sheet).expect_err("duplicate");
        assert!(matches!(error, ExcelError::Format(_)));
    }
}

#[cfg(test)]
mod tests_extra {
    use std::collections::HashMap;

    use easyexcel_core::support::ExcelTypeEnum;
    use easyexcel_core::{CellValue, CustomReadObject};

    use super::*;
    use crate::holder::read_row_holder::ReadRowHolder;

    #[test]
    fn holder_accessors_and_row_state() -> Result<()> {
        // 对应 Java：AnalysisContextImpl 各 holder 访问器
        let options = ReadOptions::default();
        let mut context = AnalysisContextImpl::new(ExcelTypeEnum::Xls, &options);

        assert!(context.read_sheet_holder().is_none());
        assert!(context.read_row_holder().is_none());
        assert!(context.read_sheet_list().is_none());
        assert!(context.custom().is_none());
        assert_eq!(context.excel_type(), ExcelTypeEnum::Xls);
        assert_eq!(context.current_row_num(), None);
        assert_eq!(
            context.read_workbook_holder().ignore_empty_row,
            options.ignore_empty_row
        );

        // current_sheet 后 sheet holder 就位
        let sheet = ReadSheet::with_name(2, "Data");
        context.current_sheet(&sheet)?;
        let holder = context.read_sheet_holder().expect("sheet holder");
        assert_eq!(holder.sheet_no, 2);
        assert_eq!(holder.sheet_name, "Data");

        // row holder 设置与读取（对应 Java：readRowHolder(ReadRowHolder)）
        let mut cells = HashMap::new();
        cells.insert(0usize, CellValue::Int(7));
        context.set_read_row_holder(ReadRowHolder::new(5, cells));
        assert_eq!(context.current_row_num(), Some(5));
        assert_eq!(context.read_row_holder().expect("row holder").row_index, 5);

        // read_sheet_list 设置与读取（对应 Java：readSheetList(List<ReadSheet>)）
        context.set_read_sheet_list(vec![ReadSheet::new(0)]);
        assert_eq!(context.read_sheet_list().expect("sheet list").len(), 1);

        // analysis_context_mut 修改自定义对象（对应 Java：getCustom()）
        *context.analysis_context_mut() = context
            .analysis_context()
            .clone()
            .with_custom_object(Some(CustomReadObject::new(9_u32)));
        assert!(context.custom().is_some());

        // analysis_event_processor 默认实现为 no-op（对应 Java：analysisEventProcessor()）
        let context_inner = context.analysis_context().clone();
        let processor = context.analysis_event_processor();
        processor.extra(&context_inner);
        processor.end_row(&context_inner);
        processor.end_sheet(&context_inner);

        // interrupt 为 Unsupported（对应 Java：@Deprecated interrupt()）
        assert!(context.interrupt().is_err());
        Ok(())
    }
}
