/// 对应 Java：无直接对应对象；Rust 架构扩展。 Cached formula result kinds aligned with POI `CellType.forInt`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormulaCachedType {
    /// String result — Java waits for the next `StringRecord`.
    String,
    /// Numeric result.
    Numeric,
    /// Error result (`#VALUE!`).
    Error,
    /// Boolean result.
    Boolean,
    /// Empty / unknown.
    Empty,
}

