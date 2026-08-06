/// Action selected by a listener after a row error.
///
/// 对应 Java：`ReadListener.onException(...)` semantics:
/// * `Continue` ⇒ Java's `onException` returns without throwing.
/// * `SkipRow` ⇒ Rust extension for batch pagination.
/// * `Stop` ⇒ Java's `onException` throws `ExcelAnalysisException`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ErrorAction {
    /// Continue with the next row.
    Continue,
    /// Skip the failed row and continue.
    SkipRow,
    /// Stop and return the error. (default — matches Java's throw-exception behaviour)
    #[default]
    Stop,
}

