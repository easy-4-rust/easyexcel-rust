//! 串行映射读取监听器。

use easyexcel::{AnalysisContext, ReadListener};

use crate::benchmark_row::BenchmarkRow;

use super::apply_benchmark_map;
use super::event_listener::EventListener;

/// 串行映射读取监听器，在将行数据转发给下游监听器前应用基准测试映射函数。
///
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) struct SerialMapListener {
    /// 下游事件监听器
    pub(super) downstream: EventListener,
    /// 工作因子，控制映射的计算量
    pub(super) work_factor: u32,
}

impl ReadListener<BenchmarkRow> for SerialMapListener {
    /// 处理一行读取数据，先应用基准测试映射再转发给下游监听器。
    ///
    /// # 参数
    /// - `data`: 读取到的行数据
    /// - `context`: 分析上下文
    fn invoke(&mut self, data: BenchmarkRow, context: &AnalysisContext) -> easyexcel::Result<()> {
        self.downstream
            .invoke(apply_benchmark_map(data, self.work_factor), context)
    }
}
