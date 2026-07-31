//! Mirrors Java `com.alibaba.excel.read.processor.DefaultAnalysisEventProcessor`.

use crate::processor::analysis_event_processor::AnalysisEventProcessor;
use easyexcel_core::AnalysisContext;

/// 默认分析事件处理器，对应 Java `DefaultAnalysisEventProcessor`。
#[derive(Debug, Clone, Default)]
pub struct DefaultAnalysisEventProcessor;

impl AnalysisEventProcessor for DefaultAnalysisEventProcessor {
    fn extra(&mut self, _analysis_context: &AnalysisContext) {
        let _ = _analysis_context;
    }
    fn end_row(&mut self, _analysis_context: &AnalysisContext) {
        let _ = _analysis_context;
    }
    fn end_sheet(&mut self, _analysis_context: &AnalysisContext) {
        let _ = _analysis_context;
    }
}
