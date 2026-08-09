//! 对应 Java：`com.alibaba.excel.event.Handler`.
//!
//! Java `Handler extends Order`. The `order()` method has a default of
//! `OrderConstant.DEFAULT_ORDER` (= 0). Rust already encodes this on
//! `WriteHandler::order()`. This module re-exports the same value as a
//! standalone trait so 1:1 Java package references resolve.

pub use super::order::Order;

/// 对应 Java：`Handler extends Order`.
///
/// `Handler` 是标记接口；`Order` 通过下方 blanket impl 自动获得，避免
/// 每个 Handler 同时手写两个 trait 才能表达 Java 的接口继承。
pub trait Handler {}

impl<T: Handler + ?Sized> Order for T {
    fn order(&self) -> i32 {
        crate::constant::order_constant::DEFAULT_ORDER
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
