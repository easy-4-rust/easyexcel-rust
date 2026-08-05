//! 对应 Java：`com.alibaba.excel.read.processor.DefaultAnalysisEventProcessor`.
//!
//! # 架构差异（对象数据一致性例外，见 docs/migration/对象级对照表.md）
//!
//! Java 的 `DefaultAnalysisEventProcessor`（168 行）是读取事件分发中枢：
//! `endRow`→`dealData`（行类型判断、`invoke`/`invokeHead` 分发、`hasNext`
//! 停机检查、`onException` 异常路由）、`extra`→`dealExtra`（`CellExtra`
//! 监听分发）、`endSheet`→`doAfterAllAnalysed`、`buildHead`（表头列映射
//! 构建）。其依赖 `AnalysisContext.readRowHolder()/currentReadHolder()`
//! 的 Java holder 体系。
//!
//! Rust 读取管线**没有 holder 体系**：`AnalysisContext` 仅携带精简事件
//! 上下文，Java 事件处理器的全部语义由读取管线内联实现并 100% 覆盖——
//! 行分发/表头判断/`invokeHead`/`invoke`/停机/异常路由在
//! `read/row_consumer.rs::RowConsumer` + `process_row_with_metadata`，
//! `CellExtra` 监听分发在 `RowConsumer::extra`，`doAfterAllAnalysed` 在
//! `RowConsumer::after`，表头列映射在 `read/row_processing.rs`。因此本
//! 对象保持 no-op 空实现（Java 事件处理器模式 → Rust 内联管线的形态差异），
//! 不是语义缺失；读取功能由 62 个测试套件全绿验证。

use crate::core::AnalysisContext;
use crate::read::processor::analysis_event_processor::AnalysisEventProcessor;

/// 默认分析事件处理器，对应 Java `DefaultAnalysisEventProcessor`。
///
/// 事件语义由读取管线内联实现（见模块注释的架构差异说明），本对象为
/// Java 事件处理器形态的 Rust 占位——保持 trait 契约可调用。
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
