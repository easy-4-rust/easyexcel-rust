/// 对应 Java：无直接对应对象；Rust 架构扩展。 Resource-owning lifecycle capability for a [`WriteContext`].
///
/// Java's `WriteContextImpl` owns the POI workbook and can therefore implement
/// `finish(boolean)` directly. Rust keeps backend resources in the writer crate,
/// so only the resource-owning adapter implements this trait. Metadata-only
/// contexts deliberately do not pretend that they can persist or close a
/// workbook.
pub trait WriteContextLifecycle: WriteContext {
    /// Persists or discards pending output and releases owned resources.
    ///
    /// `on_exception` follows Java semantics: pending workbook bytes are
    /// discarded unless `writeExcelOnException` was enabled.
    ///
    /// # Errors
    ///
    /// Returns an output, handler, finalization, or stream-close error.
    fn finish_context(&mut self, on_exception: bool) -> Result<(), ExcelError>;
}

