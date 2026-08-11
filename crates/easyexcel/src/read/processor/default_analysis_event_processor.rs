//! 对应 Java：`com.alibaba.excel.read.processor.DefaultAnalysisEventProcessor`.
//!
//! Rust 的 Holder 与用户模型采用静态类型，因此三个 Java trait 方法仍只接收
//! 轻量 `AnalysisContext`；真实行、extra、结束事件通过本类型的类型化分发方法
//! 进入 Listener。读取管线直接调用这些方法，避免出现只有同名对象、实际链路
//! 却绕过它的假兼容。

use std::collections::HashMap;

use crate::core::{AnalysisContext, CellExtra, ExcelError, ReadListener, Result};
use crate::read::processor::analysis_event_processor::AnalysisEventProcessor;
use crate::read::row_consumer::ReadFlow;

/// 默认分析事件处理器，对应 Java `DefaultAnalysisEventProcessor`。
///
/// 类型化入口承担 Java `dealData`、`dealExtra`、异常路由、`hasNext` 与
/// `doAfterAllAnalysed` 的实际生产调用。
#[derive(Debug, Clone, Default)]
pub struct DefaultAnalysisEventProcessor;

impl DefaultAnalysisEventProcessor {
    /// 分发表头事件并执行 Java `hasNext`/`onException` 路由。
    pub(crate) fn dispatch_head<T>(
        listener: &mut dyn ReadListener<T>,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<ReadFlow> {
        let result = listener.invoke_head(head, context);
        Self::dispatch_result(result, listener, context)
    }

    /// 分发已经转换完成的数据行。
    pub(crate) fn dispatch_data<T>(
        listener: &mut dyn ReadListener<T>,
        data: T,
        context: &AnalysisContext,
    ) -> Result<ReadFlow> {
        let result = listener.invoke(data, context);
        Self::dispatch_result(result, listener, context)
    }

    /// 将模型转换错误送入全部 Listener 都遵循的异常策略。
    pub(crate) fn dispatch_error<T>(
        listener: &mut dyn ReadListener<T>,
        error: ExcelError,
        context: &AnalysisContext,
    ) -> Result<ReadFlow> {
        crate::read::read_helpers::listener_error(error, listener, context)
    }

    /// 分发 comment/hyperlink/merge 等 extra 元数据。
    pub(crate) fn dispatch_extra<T>(
        listener: &mut dyn ReadListener<T>,
        extra: &CellExtra,
        context: &AnalysisContext,
    ) -> Result<ReadFlow> {
        let result = listener.extra(extra, context);
        Self::dispatch_result(result, listener, context)
    }

    /// 分发工作表结束事件。
    pub(crate) fn dispatch_end_sheet<T>(
        listener: &mut dyn ReadListener<T>,
        context: &AnalysisContext,
    ) -> Result<()> {
        listener.do_after_all_analysed(context)
    }

    fn dispatch_result<T>(
        result: Result<()>,
        listener: &mut dyn ReadListener<T>,
        context: &AnalysisContext,
    ) -> Result<ReadFlow> {
        crate::read::read_helpers::listener_result(result, listener, context)
    }
}

impl AnalysisEventProcessor for DefaultAnalysisEventProcessor {
    fn extra(&mut self, _: &AnalysisContext) {
        // 无类型的 Java 兼容入口；生产链使用 dispatch_extra。
    }

    fn end_row(&mut self, _: &AnalysisContext) {
        // 无行负载的 Java 兼容入口；生产链使用 dispatch_head/dispatch_data。
    }

    fn end_sheet(&mut self, _: &AnalysisContext) {
        // 无 Listener 的 Java 兼容入口；生产链使用 dispatch_end_sheet。
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_processor_events_are_noops() {
        // 对应 Java：DefaultAnalysisEventProcessor 三个事件默认空实现
        let mut processor = DefaultAnalysisEventProcessor;
        let context = AnalysisContext::new("Sheet1", 0, 0);
        processor.extra(&context);
        processor.end_row(&context);
        processor.end_sheet(&context);
    }
}
