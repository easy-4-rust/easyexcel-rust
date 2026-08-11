//! 事件驱动读取监听器。

use std::cell::RefCell;
use std::rc::Rc;

use easyexcel::{AnalysisContext, ReadListener};

use crate::benchmark_row::BenchmarkRow;

use super::event_state::EventState;

/// 事件驱动读取监听器，将每行数据委托给共享的 [`EventState`]。
///
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(super) struct EventListener(pub(super) Rc<RefCell<EventState>>);

impl ReadListener<BenchmarkRow> for EventListener {
    /// 处理一行读取数据，累加行计数并更新校验和。
    ///
    /// # 参数
    /// - `data`: 读取到的行数据
    /// - `_context`: 分析上下文（未使用）
    fn invoke(&mut self, data: BenchmarkRow, _context: &AnalysisContext) -> easyexcel::Result<()> {
        let mut state = self.0.borrow_mut();
        state.rows += 1;
        state.checksum.update(&data);
        Ok(())
    }
}
