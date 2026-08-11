// 分析事件监听器适配器。
// 对应 Java：`com.alibaba.excel.event.AnalysisEventListener`（适配器部分）。
// 从 `analysis_event_listener.rs` 拆分而来，
// 遵循"一个 .rs 文件只对应一个 Java 对象"规范。

/// 把 Java `AnalysisEventListener` 的基类桥接方法接入 `ReadListener` vtable。
///
/// Java 基类的 `invokeHead` 会把单元格表头转换成 `Map<Integer,String>` 后
/// 调用 `invokeHeadMap`；Rust 读取管线已经持有解析后的 name->index 映射，
/// 适配器在动态分派边界反转该映射并调用同一回调。
pub struct AnalysisEventListenerAdapter<L> {
    inner: L,
}

impl<L> AnalysisEventListenerAdapter<L> {
    /// 创建分析事件监听器适配器。
    ///
    /// # 参数
    /// - `inner`: 实现了 `AnalysisEventListener` 的监听器。
    #[must_use]
    pub const fn new(inner: L) -> Self {
        Self { inner }
    }

    /// 返回内部监听器。
    #[must_use]
    pub const fn inner(&self) -> &L {
        &self.inner
    }

    /// 返回内部监听器的可变引用。
    pub const fn inner_mut(&mut self) -> &mut L {
        &mut self.inner
    }

    /// 消费适配器并返回内部监听器。
    pub fn into_inner(self) -> L {
        self.inner
    }
}

impl<T, L> ReadListener<T> for AnalysisEventListenerAdapter<L>
where
    L: AnalysisEventListener<T>,
{
    fn on_exception(
        &mut self,
        error: &ExcelError,
        context: &crate::AnalysisContext,
    ) -> crate::core::ErrorAction {
        ReadListener::on_exception(&mut self.inner, error, context)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &crate::AnalysisContext,
    ) -> crate::Result<()> {
        let head_map = head
            .iter()
            .map(|(name, index)| (*index, name.clone()))
            .collect();
        self.inner.invoke_head_map(head_map, context);
        Ok(())
    }

    fn invoke(&mut self, data: T, context: &crate::AnalysisContext) -> crate::Result<()> {
        ReadListener::invoke(&mut self.inner, data, context)
    }

    fn extra(&mut self, extra: &CellExtra, context: &crate::AnalysisContext) -> crate::Result<()> {
        ReadListener::extra(&mut self.inner, extra, context)
    }

    fn do_after_all_analysed(&mut self, context: &crate::AnalysisContext) -> crate::Result<()> {
        ReadListener::do_after_all_analysed(&mut self.inner, context)
    }

    fn has_next(&mut self, context: &crate::AnalysisContext) -> bool {
        ReadListener::has_next(&mut self.inner, context)
    }
}
