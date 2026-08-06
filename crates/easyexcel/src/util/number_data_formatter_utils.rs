//! Java `NumberDataFormatterUtils` 兼容入口。

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按格式代码的小数位数格式化数值。
#[must_use]
pub fn format(value: f64, format_pattern: &str) -> String {
    easyexcel_model::numfmt::format_fixed_decimal(value, format_pattern)
}

/// Java `ThreadLocal<DecimalFormat>` 清理兼容钩子。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const fn remove_thread_local_cache() {
    // easyexcel-model 的数值格式化是无状态调用，不存在待清理的线程缓存。
}
