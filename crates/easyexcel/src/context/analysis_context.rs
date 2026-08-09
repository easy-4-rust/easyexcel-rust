//! 对应 Java：`com.alibaba.excel.context.AnalysisContext` (the public listener
//! surface — `AnalysisContextImpl` carries additional mutable state).

use std::any::Any;

use crate::core::custom_read_object::CustomReadObject;
use crate::core::excel_error::ExcelError;

use super::analysis_context_impl::AnalysisContextImpl;

/// Java `AnalysisContext` 的完整读取生命周期父契约。
///
/// [`AnalysisContext`] 继续作为逐行 listener 的轻量快照；该 trait 承载 Java 接口中的
/// Holder、事件处理器、输入与 interrupt 状态，使格式专用 Context 形成真实 supertrait。
pub trait AnalysisContextLifecycle {
    /// 返回共享生命周期实现。
    fn analysis_context_impl(&self) -> &AnalysisContextImpl;
    /// 返回可变共享生命周期实现。
    fn analysis_context_impl_mut(&mut self) -> &mut AnalysisContextImpl;

    /// Java `analysisEventProcessor`。
    fn analysis_event_processor(
        &mut self,
    ) -> &mut dyn crate::read::processor::analysis_event_processor::AnalysisEventProcessor {
        self.analysis_context_impl_mut().analysis_event_processor()
    }
    /// Java `currentReadHolder` 的后端中立 holder 类型。
    fn current_read_holder(&self) -> crate::HolderEnum {
        self.analysis_context_impl().current_read_holder()
    }
    /// Java `currentSheet`。
    fn current_sheet(&mut self, sheet: &super::read_sheet::ReadSheet) -> crate::Result<()> {
        self.analysis_context_impl_mut().current_sheet(sheet)
    }
    /// Java `getCurrentRowAnalysisResult`。
    fn get_current_row_analysis_result(&self) -> Option<&CustomReadObject> {
        self.analysis_context_impl().get_current_row_analysis_result()
    }
    /// Java `getCurrentRowNum`。
    fn get_current_row_num(&self) -> Option<i32> {
        self.analysis_context_impl().get_current_row_num()
    }
    /// Java `getCustom`。
    fn get_custom_object(&self) -> Option<&CustomReadObject> {
        self.analysis_context_impl().get_custom()
    }
    /// Java `getExcelType`。
    fn get_excel_type(&self) -> crate::support::ExcelTypeEnum {
        self.analysis_context_impl().excel_type()
    }
    /// Java `getInputStream` 的拥有字节视图。
    fn get_input_stream(&self) -> Option<&[u8]> {
        self.analysis_context_impl().get_input_stream()
    }
    /// Java `getTotalCount`。
    fn get_total_count(&self) -> Option<i32> {
        self.analysis_context_impl().get_total_count()
    }
    /// Java `interrupt`。
    fn interrupt(&self) -> crate::Result<()> { self.analysis_context_impl().interrupt() }
    /// Java `readRowHolder`。
    fn read_row_holder(&self) -> Option<&crate::read::holder::read_row_holder::ReadRowHolder> {
        self.analysis_context_impl().read_row_holder()
    }
    /// Java `readRowHolder(ReadRowHolder)`。
    fn set_read_row_holder(
        &mut self,
        holder: crate::read::holder::read_row_holder::ReadRowHolder,
    ) {
        self.analysis_context_impl_mut().set_read_row_holder(holder);
    }
    /// Java `readSheetHolder`。
    fn read_sheet_holder(&self) -> Option<&crate::read::holder::read_sheet_holder::ReadSheetHolder> {
        self.analysis_context_impl().read_sheet_holder()
    }
    /// Java `readSheetList`。
    fn read_sheet_list(&self) -> Option<&[super::read_sheet::ReadSheet]> {
        self.analysis_context_impl().read_sheet_list()
    }
    /// Java `readSheetList(List)`。
    fn set_read_sheet_list(&mut self, sheets: Vec<super::read_sheet::ReadSheet>) {
        self.analysis_context_impl_mut().set_read_sheet_list(sheets);
    }
    /// Java `readWorkbookHolder`。
    fn read_workbook_holder(
        &self,
    ) -> &crate::read::holder::read_workbook_holder::ReadWorkbookHolder {
        self.analysis_context_impl().read_workbook_holder()
    }
}

/// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Read callback context equivalent to Java `AnalysisContext`.
///
/// Java exposes 14 methods plus several `@Deprecated` accessors. Rust keeps
/// only the methods actually consumed by `ReadListener` callbacks; legacy
/// getters (`getExcelType`, `getInputStream`, `getCurrentRowNum`, etc.) are
/// replaced by fields carried elsewhere in the read pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalysisContext {
    sheet_name: String,
    sheet_no: usize,
    row_index: u32,
    batch_index: usize,
    custom_object: Option<CustomReadObject>,
}

impl AnalysisContext {
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Creates a context. (Java `AnalysisContextImpl(ReadWorkbook, ExcelTypeEnum)` initial state)
    #[must_use]
    pub fn new(sheet_name: impl Into<String>, sheet_no: usize, row_index: u32) -> Self {
        Self {
            sheet_name: sheet_name.into(),
            sheet_no,
            row_index,
            batch_index: 0,
            custom_object: None,
        }
    }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns the sheet name. (Java `AnalysisContext.readSheetHolder().getSheetName()`)
    #[must_use]
    pub fn sheet_name(&self) -> &str {
        &self.sheet_name
    }

    /// Returns the zero-based sheet index. (Java `AnalysisContext.readSheetHolder().getSheetNo()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn sheet_no(&self) -> usize {
        self.sheet_no
    }

    /// Returns the zero-based physical row index. (Java `getCurrentRowNum()`)
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn row_index(&self) -> u32 {
        self.row_index
    }

    /// Returns the zero-based callback batch index.
    /// Rust extension tracking the page index in `PageReadListener`.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn batch_index(&self) -> usize {
        self.batch_index
    }

    /// Returns the configured custom read object, if any.
    #[must_use]
    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。
    pub const fn custom_object(&self) -> Option<&CustomReadObject> {
        self.custom_object.as_ref()
    }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns the custom read object when its concrete type matches `T`.
    /// Mirrors `(T) AnalysisContext.getCustom()` after an explicit cast.
    #[must_use]
    pub fn custom<T: Any>(&self) -> Option<&T> {
        self.custom_object.as_ref()?.downcast_ref()
    }
    /// Java `getCustom()` 的类型安全兼容入口。
    #[must_use]
    pub fn get_custom<T: Any>(&self) -> Option<&T> { self.custom() }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns a context carrying the supplied custom read object.
    #[must_use]
    pub fn with_custom_object(mut self, custom_object: Option<CustomReadObject>) -> Self {
        self.custom_object = custom_object;
        self
    }

    /// 对应 Java：com.alibaba.excel.context.AnalysisContext。 Returns a copy with a different batch index.
    #[must_use]
    pub fn with_batch_index(&self, batch_index: usize) -> Self {
        let mut context = self.clone();
        context.batch_index = batch_index;
        context
    }
}

include!("analysis_context/error_action.rs");

include!("analysis_context/result.rs");
