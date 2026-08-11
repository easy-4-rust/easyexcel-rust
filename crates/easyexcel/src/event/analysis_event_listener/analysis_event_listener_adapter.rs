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

#[cfg(test)]
mod adapter_tests {
    use super::*;
    use std::cell::RefCell;

    /// 测试用监听器，记录回调次数。
    struct TestListener {
        invoke_count: RefCell<usize>,
        head_map_count: RefCell<usize>,
        after_count: RefCell<usize>,
    }

    impl TestListener {
        fn new() -> Self {
            Self {
                invoke_count: RefCell::new(0),
                head_map_count: RefCell::new(0),
                after_count: RefCell::new(0),
            }
        }
    }

    impl crate::ReadListener<crate::CellValue> for TestListener {
        fn invoke(
            &mut self,
            _data: crate::CellValue,
            _context: &crate::AnalysisContext,
        ) -> crate::Result<()> {
            *self.invoke_count.borrow_mut() += 1;
            Ok(())
        }

        fn do_after_all_analysed(
            &mut self,
            _context: &crate::AnalysisContext,
        ) -> crate::Result<()> {
            *self.after_count.borrow_mut() += 1;
            Ok(())
        }
    }

    impl crate::event::AnalysisEventListener<crate::CellValue> for TestListener {
        fn invoke_head_map(
            &mut self,
            _head_map: std::collections::HashMap<usize, String>,
            _context: &crate::AnalysisContext,
        ) {
            *self.head_map_count.borrow_mut() += 1;
        }
    }

    #[test]
    fn new_creates_adapter() {
        let listener = TestListener::new();
        let adapter = AnalysisEventListenerAdapter::new(listener);
        assert_eq!(*adapter.inner().invoke_count.borrow(), 0);
    }

    #[test]
    fn inner_returns_reference() {
        let listener = TestListener::new();
        let adapter = AnalysisEventListenerAdapter::new(listener);
        let _inner: &TestListener = adapter.inner();
    }

    #[test]
    fn inner_mut_returns_mutable_reference() {
        let listener = TestListener::new();
        let mut adapter = AnalysisEventListenerAdapter::new(listener);
        let _inner: &mut TestListener = adapter.inner_mut();
    }

    #[test]
    fn into_inner_consumes_adapter() {
        let listener = TestListener::new();
        let adapter = AnalysisEventListenerAdapter::new(listener);
        let inner = adapter.into_inner();
        assert_eq!(*inner.invoke_count.borrow(), 0);
    }

    #[test]
    fn read_listener_invoke_delegates() {
        let listener = TestListener::new();
        let mut adapter = AnalysisEventListenerAdapter::new(listener);
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        adapter
            .invoke(crate::CellValue::Int(1), &context)
            .unwrap();
        assert_eq!(*adapter.inner().invoke_count.borrow(), 1);
    }

    #[test]
    fn read_listener_do_after_all_analysed_delegates() {
        let listener = TestListener::new();
        let mut adapter = AnalysisEventListenerAdapter::new(listener);
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        adapter.do_after_all_analysed(&context).unwrap();
        assert_eq!(*adapter.inner().after_count.borrow(), 1);
    }

    #[test]
    fn invoke_head_converts_map_and_calls_invoke_head_map() {
        let listener = TestListener::new();
        let mut adapter = AnalysisEventListenerAdapter::new(listener);
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        let mut head = std::collections::HashMap::new();
        head.insert("Name".to_owned(), 0);
        head.insert("Age".to_owned(), 1);
        adapter.invoke_head(&head, &context).unwrap();
        assert_eq!(*adapter.inner().head_map_count.borrow(), 1);
    }

    #[test]
    fn has_next_delegates_to_inner() {
        let listener = TestListener::new();
        let mut adapter = AnalysisEventListenerAdapter::new(listener);
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        // 默认 ReadListener::has_next 返回 true
        assert!(adapter.has_next(&context));
    }

    #[test]
    fn on_exception_delegates_to_inner() {
        let listener = TestListener::new();
        let mut adapter = AnalysisEventListenerAdapter::new(listener);
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        let error = crate::ExcelError::Format("test".to_owned());
        let action = adapter.on_exception(&error, &context);
        // 默认 ReadListener::on_exception 返回 Stop
        assert_eq!(action, crate::core::ErrorAction::Stop);
    }

    #[test]
    fn extra_delegates_to_inner() {
        let listener = TestListener::new();
        let mut adapter = AnalysisEventListenerAdapter::new(listener);
        let context = crate::AnalysisContext::new("Sheet1", 0, 0);
        let extra = crate::CellExtra::new(
            crate::core::enum_cell_extra_type::CellExtraType::Merge,
            None, 0, 1, 0, 1,
        );
        adapter.extra(&extra, &context).unwrap();
    }
}
