//! 对应 Java：`com.alibaba.excel.event.NotRepeatExecutor`.

/// There are multiple interceptors that execute only one of them when
/// fired. If you want to control which one to execute please use
/// [`Order`](crate::event::order::Order).
///
/// Rust port of Java `NotRepeatExecutor`.
/// 对应 Java：com.alibaba.excel.event.NotRepeatExecutor。
pub trait NotRepeatExecutor {
    /// Returns a unique string identifying this executor so deduplication
    /// can skip repeats. (Java `uniqueValue()`)
    fn unique_value(&self) -> &str;
}
