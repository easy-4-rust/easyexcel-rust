// 读取生命周期父契约 trait。
// 对应 Java：`com.alibaba.excel.context.AnalysisContext`（完整接口层）。
// 从 `analysis_context.rs` 拆分而来，遵循"一个 .rs 文件只对应一个 Java 对象"规范。
//
// `AnalysisContextLifecycle` 继承 `AnalysisContext` 的轻量快照角色，
// 承载 Java 接口中的 Holder、事件处理器、输入与 interrupt 状态，
// 使格式专用 Context 形成真实 supertrait。

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
