//! 对应 Java：`com.alibaba.excel.event.Handler`.
//!
//! Java `Handler extends Order`. The `order()` method has a default of
//! `OrderConstant.DEFAULT_ORDER` (= 0). Rust already encodes this on
//! `WriteHandler::order()`. This module re-exports the same value as a
//! standalone trait so 1:1 Java package references resolve.

/// 对应 Java：`Handler extends Order`.
///
/// `Handler` is a marker extension of `Order`; Rust mirrors the
/// contract through the `order()` method.
pub trait Handler {
    /// Returns the handler's execution order. Lower values execute first.
    /// (Java `Handler.order()` defaulting to `OrderConstant.DEFAULT_ORDER`)
    fn order(&self) -> i32 {
        0
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    struct ProbeHandler;

    impl Handler for ProbeHandler {}

    #[test]
    fn default_order_is_zero() {
        // 对应 Java：Handler.order() 默认 OrderConstant.DEFAULT_ORDER
        assert_eq!(ProbeHandler.order(), 0);
        let probe = ProbeHandler;
        assert_eq!(probe.order(), 0);
    }
}
