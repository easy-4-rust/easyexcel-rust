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

#[cfg(test)]
mod lifecycle_tests {
    use super::*;

    /// 创建一个已初始化的 `AnalysisContextImpl` 用于测试。
    fn make_impl() -> super::super::analysis_context_impl::AnalysisContextImpl {
        let options = crate::ReadOptions::default();
        super::super::analysis_context_impl::AnalysisContextImpl::new(
            crate::support::ExcelTypeEnum::Xlsx,
            &options,
        )
    }

    #[test]
    fn analysis_context_impl_returns_ref() {
        let ctx = make_impl();
        let _impl_ref: &super::super::analysis_context_impl::AnalysisContextImpl =
            ctx.analysis_context_impl();
    }

    #[test]
    fn analysis_context_impl_mut_returns_mut_ref() {
        let mut ctx = make_impl();
        let _impl_ref: &mut super::super::analysis_context_impl::AnalysisContextImpl =
            ctx.analysis_context_impl_mut();
    }

    #[test]
    fn current_read_holder_returns_workbook_when_no_sheet() {
        let ctx = make_impl();
        let holder = ctx.current_read_holder();
        assert_eq!(holder, crate::HolderEnum::Workbook);
    }

    #[test]
    fn get_excel_type_returns_xlsx() {
        let ctx = make_impl();
        assert_eq!(ctx.get_excel_type(), crate::support::ExcelTypeEnum::Xlsx);
    }

    #[test]
    fn get_input_stream_returns_none_without_input() {
        let ctx = make_impl();
        assert!(ctx.get_input_stream().is_none());
    }

    #[test]
    fn get_current_row_num_returns_none_without_row() {
        let ctx = make_impl();
        assert!(ctx.get_current_row_num().is_none());
    }

    #[test]
    fn get_total_count_returns_none_without_sheet() {
        let ctx = make_impl();
        assert!(ctx.get_total_count().is_none());
    }

    #[test]
    fn get_custom_object_returns_none() {
        let ctx = make_impl();
        assert!(ctx.get_custom_object().is_none());
    }

    #[test]
    fn get_current_row_analysis_result_returns_none() {
        let ctx = make_impl();
        assert!(ctx.get_current_row_analysis_result().is_none());
    }

    #[test]
    fn read_row_holder_returns_none_initially() {
        let ctx = make_impl();
        assert!(ctx.read_row_holder().is_none());
    }

    #[test]
    fn read_sheet_holder_returns_none_initially() {
        let ctx = make_impl();
        assert!(ctx.read_sheet_holder().is_none());
    }

    #[test]
    fn read_sheet_list_returns_none_initially() {
        let ctx = make_impl();
        assert!(ctx.read_sheet_list().is_none());
    }

    #[test]
    fn interrupt_returns_error() {
        let ctx = make_impl();
        assert!(ctx.interrupt().is_err());
    }

    #[test]
    fn read_workbook_holder_returns_ref() {
        let ctx = make_impl();
        let _holder: &crate::read::holder::read_workbook_holder::ReadWorkbookHolder =
            ctx.read_workbook_holder();
    }

    #[test]
    fn current_sheet_sets_sheet() {
        let mut ctx = make_impl();
        let sheet = super::super::read_sheet::ReadSheet::with_name(0, "Sheet1");
        ctx.current_sheet(&sheet).unwrap();
        assert_eq!(ctx.current_read_holder(), crate::HolderEnum::Sheet);
    }

    #[test]
    fn set_read_sheet_list_updates() {
        let mut ctx = make_impl();
        ctx.set_read_sheet_list(vec![
            super::super::read_sheet::ReadSheet::new(0),
        ]);
        assert!(ctx.read_sheet_list().is_some());
        assert_eq!(ctx.read_sheet_list().unwrap().len(), 1);
    }
}
