//! 对应 Java：`com.alibaba.excel.read.processor.DefaultAnalysisEventProcessor`.

use crate::core::AnalysisContext;
use crate::read::processor::analysis_event_processor::AnalysisEventProcessor;

/// 默认分析事件处理器，对应 Java `DefaultAnalysisEventProcessor`。
#[derive(Debug, Clone, Default)]
pub struct DefaultAnalysisEventProcessor;

impl AnalysisEventProcessor for DefaultAnalysisEventProcessor {
    fn extra(&mut self, _: &AnalysisContext) {}
    fn end_row(&mut self, _: &AnalysisContext) {}
    fn end_sheet(&mut self, _: &AnalysisContext) {}
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
