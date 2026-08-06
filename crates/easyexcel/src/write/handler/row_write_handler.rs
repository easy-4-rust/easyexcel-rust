//! 对应 Java：`com.alibaba.excel.write.handler.RowWriteHandler`.

/// Marks a handler as the Rust counterpart of Java `RowWriteHandler`.
///
/// Implement `before_row_create`, `after_row_create`, and
/// `after_row_dispose` on [`crate::core::WriteHandler`].
/// 对应 Java：com.alibaba.excel.write.handler.RowWriteHandler。
pub trait RowWriteHandler: crate::core::WriteHandler {}
