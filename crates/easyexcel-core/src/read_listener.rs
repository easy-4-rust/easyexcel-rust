//! Mirrors Java `com.alibaba.excel.read.listener.ReadListener<T>` (and the
//! `IgnoreExceptionReadListener` default implementation).

use std::collections::HashMap;

use crate::analysis_context::{AnalysisContext, ErrorAction, Result};
use crate::cell_extra::CellExtra;
use crate::excel_error::ExcelError;

/// Event listener equivalent to Java `EasyExcel`'s `ReadListener`.
///
/// Java `ReadListener` is an interface with one abstract method (`invoke`).
/// Rust keeps the same shape: `invoke` is the only required method; the
/// other four callbacks have default no-op implementations.
pub trait ReadListener<T> {
    /// Called when row conversion or processing fails.
    ///
    /// Mirrors Java `onException(Exception, AnalysisContext) throws Exception`,
    /// where the exception is mapped to [`ErrorAction`].
    fn on_exception(&mut self, _error: &ExcelError, _context: &AnalysisContext) -> ErrorAction {
        ErrorAction::Stop
    }

    /// Called for a resolved header row. (Java `invokeHead(Map<Integer, ReadCellData<?>>, AnalysisContext)`)
    ///
    /// # Errors
    ///
    /// Returns an error to stop the read operation.
    fn invoke_head(
        &mut self,
        _head: &HashMap<String, usize>,
        _context: &AnalysisContext,
    ) -> Result<()> {
        Ok(())
    }

    /// Called once for every successfully converted row. (Java `invoke(T, AnalysisContext)`)
    ///
    /// # Errors
    ///
    /// Returns an error to stop the read operation.
    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()>;

    /// Called when enabled comment, hyperlink, or merge metadata is encountered.
    /// (Java `extra(CellExtra, AnalysisContext)`)
    ///
    /// # Errors
    ///
    /// Returns an error to route through [`Self::on_exception`].
    fn extra(&mut self, _extra: &CellExtra, _context: &AnalysisContext) -> Result<()> {
        Ok(())
    }

    /// Called after a sheet has been analysed. (Java `doAfterAllAnalysed(AnalysisContext)`)
    ///
    /// # Errors
    ///
    /// Returns an error when final listener work fails.
    fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> Result<()> {
        Ok(())
    }

    /// Allows a listener to stop before the next row. (Java `hasNext(AnalysisContext)`)
    fn has_next(&mut self, _context: &AnalysisContext) -> bool {
        true
    }
}

/// Dispatches every read callback to two listeners in registration order.
///
/// Java stores a list of custom `ReadListener`s on `ReadBasicParameter`.
/// Rust models the same ordered fan-out as a nested, statically typed listener
/// so registering another listener does not require runtime type erasure.
pub struct CompositeReadListener<T, First, Second> {
    first: First,
    second: Second,
    marker: std::marker::PhantomData<fn() -> T>,
}

impl<T, First, Second> CompositeReadListener<T, First, Second> {
    /// Creates an ordered pair where `first` is invoked before `second`.
    #[must_use]
    pub const fn new(first: First, second: Second) -> Self {
        Self {
            first,
            second,
            marker: std::marker::PhantomData,
        }
    }

    /// Returns both listeners after a read completes.
    #[must_use]
    pub fn into_inner(self) -> (First, Second) {
        (self.first, self.second)
    }
}

/// Ordered, dynamically sized Java-style custom listener list.
///
/// This is used by compatibility builders whose listener count is only known
/// at runtime. Rows are cloned because Rust listeners own their argument,
/// while Java listeners receive the same object reference.
pub struct ReadListenerList<T> {
    listeners: Vec<Box<dyn ReadListener<T>>>,
}

impl<T> Default for ReadListenerList<T> {
    fn default() -> Self {
        Self {
            listeners: Vec::new(),
        }
    }
}

impl<T> ReadListenerList<T> {
    /// Creates a list containing its first listener.
    #[must_use]
    pub fn new(listener: impl ReadListener<T> + 'static) -> Self {
        Self {
            listeners: vec![Box::new(listener)],
        }
    }

    /// Appends a listener in Java registration order.
    pub fn push(&mut self, listener: impl ReadListener<T> + 'static) {
        self.listeners.push(Box::new(listener));
    }

    /// Appends an already boxed listener.
    pub fn push_boxed(&mut self, listener: Box<dyn ReadListener<T>>) {
        self.listeners.push(listener);
    }

    /// Returns the registered listener count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.listeners.len()
    }

    /// Returns whether no listeners are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.listeners.is_empty()
    }
}

impl<T> ReadListener<T> for ReadListenerList<T>
where
    T: Clone,
{
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        self.listeners
            .iter_mut()
            .map(|listener| listener.on_exception(error, context))
            .fold(ErrorAction::Continue, strongest_error_action)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        for listener in &mut self.listeners {
            listener.invoke_head(head, context)?;
        }
        Ok(())
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        for listener in &mut self.listeners {
            listener.invoke(data.clone(), context)?;
        }
        Ok(())
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        for listener in &mut self.listeners {
            listener.extra(extra, context)?;
        }
        Ok(())
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        for listener in &mut self.listeners {
            listener.do_after_all_analysed(context)?;
        }
        Ok(())
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        let mut has_next = true;
        for listener in &mut self.listeners {
            has_next &= listener.has_next(context);
        }
        has_next
    }
}

const fn strongest_error_action(left: ErrorAction, right: ErrorAction) -> ErrorAction {
    match (left, right) {
        (ErrorAction::Stop, _) | (_, ErrorAction::Stop) => ErrorAction::Stop,
        (ErrorAction::SkipRow, _) | (_, ErrorAction::SkipRow) => ErrorAction::SkipRow,
        _ => ErrorAction::Continue,
    }
}

impl<T, First, Second> ReadListener<T> for CompositeReadListener<T, First, Second>
where
    T: Clone,
    First: ReadListener<T>,
    Second: ReadListener<T>,
{
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        let first_action = self.first.on_exception(error, context);
        let second_action = self.second.on_exception(error, context);
        strongest_error_action(first_action, second_action)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        self.first.invoke_head(head, context)?;
        self.second.invoke_head(head, context)
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        self.first.invoke(data.clone(), context)?;
        self.second.invoke(data, context)
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        self.first.extra(extra, context)?;
        self.second.extra(extra, context)
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        self.first.do_after_all_analysed(context)?;
        self.second.do_after_all_analysed(context)
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        let first_has_next = self.first.has_next(context);
        let second_has_next = self.second.has_next(context);
        first_has_next && second_has_next
    }
}

impl<T, L: ReadListener<T> + ?Sized> ReadListener<T> for Box<L> {
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        (**self).on_exception(error, context)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        (**self).invoke_head(head, context)
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        (**self).invoke(data, context)
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        (**self).extra(extra, context)
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        (**self).do_after_all_analysed(context)
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        (**self).has_next(context)
    }
}

impl<T, L: ReadListener<T> + ?Sized> ReadListener<T> for &mut L {
    fn on_exception(&mut self, error: &ExcelError, context: &AnalysisContext) -> ErrorAction {
        (**self).on_exception(error, context)
    }

    fn invoke_head(
        &mut self,
        head: &HashMap<String, usize>,
        context: &AnalysisContext,
    ) -> Result<()> {
        (**self).invoke_head(head, context)
    }

    fn invoke(&mut self, data: T, context: &AnalysisContext) -> Result<()> {
        (**self).invoke(data, context)
    }

    fn extra(&mut self, extra: &CellExtra, context: &AnalysisContext) -> Result<()> {
        (**self).extra(extra, context)
    }

    fn do_after_all_analysed(&mut self, context: &AnalysisContext) -> Result<()> {
        (**self).do_after_all_analysed(context)
    }

    fn has_next(&mut self, context: &AnalysisContext) -> bool {
        (**self).has_next(context)
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    /// 对应 Java：测试用 ReadListener，记录回调次数并返回配置的错误动作。
    #[derive(Default)]
    struct ProbeListener {
        invokes: usize,
        heads: usize,
        extras: usize,
        afters: usize,
        has_next_result: bool,
        on_exception_action: ErrorAction,
        error_seen: Option<String>,
    }

    impl ProbeListener {
        fn with_error_action(action: ErrorAction) -> Self {
            Self {
                on_exception_action: action,
                ..Self::default()
            }
        }
    }

    impl ReadListener<i32> for ProbeListener {
        fn on_exception(&mut self, error: &ExcelError, _context: &AnalysisContext) -> ErrorAction {
            self.error_seen = Some(error.to_string());
            self.on_exception_action
        }

        fn invoke_head(
            &mut self,
            _head: &HashMap<String, usize>,
            _context: &AnalysisContext,
        ) -> Result<()> {
            self.heads += 1;
            Ok(())
        }

        fn invoke(&mut self, _data: i32, _context: &AnalysisContext) -> Result<()> {
            self.invokes += 1;
            Ok(())
        }

        fn extra(&mut self, _extra: &CellExtra, _context: &AnalysisContext) -> Result<()> {
            self.extras += 1;
            Ok(())
        }

        fn do_after_all_analysed(&mut self, _context: &AnalysisContext) -> Result<()> {
            self.afters += 1;
            Ok(())
        }

        fn has_next(&mut self, _context: &AnalysisContext) -> bool {
            self.has_next_result
        }
    }

    fn sample_context() -> AnalysisContext {
        AnalysisContext::new("Sheet1", 0, 1)
    }

    fn sample_head() -> HashMap<String, usize> {
        HashMap::from([("name".to_string(), 0)])
    }

    fn sample_extra() -> CellExtra {
        CellExtra::new(
            crate::enum_cell_extra_type::CellExtraType::Comment,
            Some("note".to_string()),
            0,
            0,
            1,
            1,
        )
    }

    #[test]
    fn composite_read_listener_into_inner_returns_pair() {
        // 对应 Java：CompositeReadListener 拆分为两个监听器
        let first = ProbeListener::default();
        let second = ProbeListener::default();
        let composite: CompositeReadListener<i32, ProbeListener, ProbeListener> =
            CompositeReadListener::new(first, second);
        let (first, second) = composite.into_inner();
        assert_eq!(first.invokes, 0);
        assert_eq!(second.invokes, 0);
    }

    #[test]
    fn read_listener_list_default_push_len_is_empty() {
        // 对应 Java：自定义监听器列表的增删查
        let mut list: ReadListenerList<i32> = ReadListenerList::default();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        list.push(ProbeListener::default());
        assert!(!list.is_empty());
        assert_eq!(list.len(), 1);
        list.push_boxed(Box::new(ProbeListener::default()));
        assert_eq!(list.len(), 2);
        let single = ReadListenerList::new(ProbeListener::default());
        assert_eq!(single.len(), 1);
    }

    #[test]
    fn read_listener_list_fans_out_callbacks() {
        // 对应 Java：ReadListenerList 按注册顺序分发所有回调
        let mut list = ReadListenerList::new(ProbeListener {
            has_next_result: true,
            ..ProbeListener::default()
        });
        list.push(ProbeListener {
            has_next_result: true,
            ..ProbeListener::default()
        });
        let context = sample_context();
        list.invoke_head(&sample_head(), &context).expect("head ok");
        list.invoke(7, &context).expect("invoke ok");
        list.extra(&sample_extra(), &context).expect("extra ok");
        list.do_after_all_analysed(&context).expect("after ok");
        let inner: &mut ReadListenerList<i32> = &mut list;
        assert!(ReadListener::has_next(inner, &context));
    }

    #[test]
    fn read_listener_list_on_exception_combines_actions() {
        // 对应 Java：多个监听器 onException 动作合并为最强动作
        let context = sample_context();
        let error = ExcelError::Format("boom".to_string());
        let mut list =
            ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::Continue));
        list.push(ProbeListener::with_error_action(ErrorAction::Continue));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::Continue);

        let mut list = ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::Stop));
        list.push(ProbeListener::with_error_action(ErrorAction::Continue));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::Stop);

        let mut list =
            ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::Continue));
        list.push(ProbeListener::with_error_action(ErrorAction::Stop));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::Stop);

        let mut list =
            ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::SkipRow));
        list.push(ProbeListener::with_error_action(ErrorAction::Continue));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::SkipRow);

        let mut list =
            ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::Continue));
        list.push(ProbeListener::with_error_action(ErrorAction::SkipRow));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::SkipRow);

        let mut list = ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::Stop));
        list.push(ProbeListener::with_error_action(ErrorAction::SkipRow));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::Stop);

        let mut list =
            ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::SkipRow));
        list.push(ProbeListener::with_error_action(ErrorAction::Stop));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::Stop);

        let mut list =
            ReadListenerList::new(ProbeListener::with_error_action(ErrorAction::SkipRow));
        list.push(ProbeListener::with_error_action(ErrorAction::SkipRow));
        assert_eq!(list.on_exception(&error, &context), ErrorAction::SkipRow);

        // 错误信息确实转发到了监听器
        let mut list = ReadListenerList::new(ProbeListener::default());
        list.on_exception(&error, &context);
        let inner: &mut ReadListenerList<i32> = &mut list;
        assert!(ReadListener::on_exception(inner, &error, &context) == ErrorAction::Stop);
    }

    #[test]
    fn composite_read_listener_on_exception_and_extra() {
        // 对应 Java：CompositeReadListener 先 first 后 second
        let context = sample_context();
        let error = ExcelError::Format("composite".to_string());
        let first = ProbeListener::with_error_action(ErrorAction::SkipRow);
        let second = ProbeListener::with_error_action(ErrorAction::Stop);
        let mut composite: CompositeReadListener<i32, ProbeListener, ProbeListener> =
            CompositeReadListener::new(first, second);
        assert_eq!(composite.on_exception(&error, &context), ErrorAction::Stop);
        composite
            .extra(&sample_extra(), &context)
            .expect("extra ok");
        assert_eq!(composite.first.extras, 1);
        assert_eq!(composite.second.extras, 1);
    }

    #[test]
    fn boxed_and_mut_ref_listeners_delegate() {
        // 对应 Java：Box<L> 与 &mut L 实现 ReadListener 委托
        let context = sample_context();
        let error = ExcelError::Format("boxed".to_string());
        let mut boxed: Box<ProbeListener> =
            Box::new(ProbeListener::with_error_action(ErrorAction::Continue));
        assert_eq!(
            ReadListener::on_exception(&mut boxed, &error, &context),
            ErrorAction::Continue
        );
        boxed.extra(&sample_extra(), &context).expect("extra ok");

        let mut probe = ProbeListener::default();
        let mut reference = &mut probe;
        assert_eq!(
            ReadListener::on_exception(&mut reference, &error, &context),
            ErrorAction::Stop
        );
        ReadListener::extra(&mut reference, &sample_extra(), &context).expect("extra ok");
        assert_eq!(probe.extras, 1);
        assert!(probe.error_seen.is_some());
    }
}
