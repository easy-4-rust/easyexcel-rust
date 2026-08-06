//! Java package-path alias for the real row handler context.

/// The Java-compatible name resolves to the runtime callback type.
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type RowWriteHandlerContext = crate::core::WriteRowContext;
