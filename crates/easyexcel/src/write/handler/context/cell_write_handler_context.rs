//! Java package-path alias for the real cell handler context.

/// The Java-compatible name resolves to the same type delivered to
/// [`crate::core::WriteHandler`] callbacks.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type CellWriteHandlerContext = crate::core::WriteCellContext;
