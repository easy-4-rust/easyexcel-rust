//! Java package-path alias for the real cell handler context.

/// The Java-compatible name resolves to the same type delivered to
/// [`crate::core::WriteHandler`] callbacks.
pub type CellWriteHandlerContext = crate::core::WriteCellContext;
