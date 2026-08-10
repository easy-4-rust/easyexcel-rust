//! XLSX 行事件读取的一次性分派计划。

use crate::read::row_consumer::RowConsumer;

/// 在进入 XML cell 循环前固定消费者需要的逐行元数据。
///
/// 对应 Java：无直接对应对象；Rust 性能扩展。普通强类型 Listener 不需要
/// Java `Map<Integer, CellData>` 的”显式存在列”集合；动态行仍完整保留它。
pub(crate) struct ReadDispatchPlan {
    retain_present_columns: bool,
    retain_formulas: bool,
    retain_display_values: bool,
    retain_decimal_values: bool,
    /// 当强类型消费者不需要 formulas/display/decimal/present 元数据时为 `true`。
    /// 配合 extras 为空，可跳过 `SourceRowMetadata` 装配直接走轻量 `process_fast`。
    typed_scalar_fast_path: bool,
}

impl ReadDispatchPlan {
    /// 根据最终消费者能力生成计划。
    #[must_use]
    pub(crate) fn compile(consumer: &dyn RowConsumer) -> Self {
        let retain_present_columns = consumer.requires_present_columns();
        let retain_formulas = consumer.requires_formulas();
        let retain_display_values = consumer.requires_display_values();
        let retain_decimal_values = consumer.requires_decimal_values();
        Self {
            typed_scalar_fast_path: !retain_present_columns
                && !retain_formulas
                && !retain_display_values
                && !retain_decimal_values,
            retain_present_columns,
            retain_formulas,
            retain_display_values,
            retain_decimal_values,
        }
    }

    /// 是否需要为每一行维护显式列集合。
    #[must_use]
    pub(crate) const fn retain_present_columns(&self) -> bool {
        self.retain_present_columns
    }

    /// 是否需要收集公式元数据。
    #[must_use]
    pub(crate) const fn retain_formulas(&self) -> bool {
        self.retain_formulas
    }

    /// 是否需要收集显示值。
    #[must_use]
    pub(crate) const fn retain_display_values(&self) -> bool {
        self.retain_display_values
    }

    /// 是否需要收集精确 decimal 值。
    #[must_use]
    pub(crate) const fn retain_decimal_values(&self) -> bool {
        self.retain_decimal_values
    }

    /// 当所有元数据收集均为 `false` 时为 `true`，表示可跳过 `SourceRowMetadata` 装配。
    #[must_use]
    pub(crate) const fn typed_scalar_fast_path(&self) -> bool {
        self.typed_scalar_fast_path
    }
}
