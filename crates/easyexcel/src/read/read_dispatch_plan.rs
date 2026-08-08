//! XLSX 行事件读取的一次性分派计划。

use crate::read::row_consumer::RowConsumer;

/// 在进入 XML cell 循环前固定消费者需要的逐行元数据。
///
/// 对应 Java：无直接对应对象；Rust 性能扩展。普通强类型 Listener 不需要
/// Java `Map<Integer, CellData>` 的“显式存在列”集合；动态行仍完整保留它。
pub(crate) struct ReadDispatchPlan {
    retain_present_columns: bool,
}

impl ReadDispatchPlan {
    /// 根据最终消费者能力生成计划。
    #[must_use]
    pub(crate) fn compile(consumer: &dyn RowConsumer) -> Self {
        Self {
            retain_present_columns: consumer.requires_present_columns(),
        }
    }

    /// 是否需要为每一行维护显式列集合。
    #[must_use]
    pub(crate) const fn retain_present_columns(&self) -> bool {
        self.retain_present_columns
    }
}
