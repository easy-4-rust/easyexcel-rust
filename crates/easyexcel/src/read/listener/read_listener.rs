//! 对应 Java：`com.alibaba.excel.read.listener.ReadListener<T>` (and the
//! `IgnoreExceptionReadListener` default implementation).
//!
//! 本文件经 `#[cfg_attr(test, mockall::automock)]` 在测试构建生成
//! `MockReadListener<T>`——automock 展开代码会引用 trait 默认方法的
//! 下划线参数（`_error`/`_head` 等），故豁免 `used_underscore_binding`。
#![allow(clippy::used_underscore_binding)]

use std::collections::HashMap;

use crate::core::analysis_context::{AnalysisContext, ErrorAction, Result};
use crate::core::cell_extra::CellExtra;
use crate::core::excel_error::ExcelError;

/// 对应 Java：com.alibaba.excel.read.listener.`ReadListener<T>`。 Event listener equivalent to Java `EasyExcel`'s `ReadListener`.
///
/// Java `ReadListener` is an interface with one abstract method (`invoke`).
/// Rust keeps the same shape: `invoke` is the only required method; the
/// other four callbacks have default no-op implementations.
///
/// 测试时经 `mockall::automock` 生成 `MockReadListener<T>`（仅 test 构建），
/// 用于断言读取管线的回调契约（调用次数/顺序/参数）——见
/// `read/row_consumer.rs` 的 `mockall_contract_tests`。
#[cfg_attr(test, mockall::automock)]
pub trait ReadListener<T> {
    /// Called when row conversion or processing fails.
    ///
    /// 对应 Java：`onException(Exception, AnalysisContext) throws Exception`,
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

include!("read_listener/composite_read_listener.rs");

include!("read_listener/read_listener_list.rs");

const fn strongest_error_action(left: ErrorAction, right: ErrorAction) -> ErrorAction {
    match (left, right) {
        (ErrorAction::Stop, _) | (_, ErrorAction::Stop) => ErrorAction::Stop,
        (ErrorAction::SkipRow, _) | (_, ErrorAction::SkipRow) => ErrorAction::SkipRow,
        _ => ErrorAction::Continue,
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
            crate::core::enum_cell_extra_type::CellExtraType::Comment,
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
        assert_eq!(
            ReadListener::on_exception(inner, &error, &context),
            ErrorAction::Stop
        );
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
