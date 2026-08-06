/// 对应 Java：无直接对应对象；Rust 架构扩展。 Decoded formula cell produced by [`FormulaRecordHandler`].
#[derive(Debug, Clone, PartialEq)]
pub struct FormulaCell {
    /// Zero-based row.
    pub row: u32,
    /// Zero-based column.
    pub column: usize,
    /// Formula text (may be `None` when parsing failed).
    pub formula: Option<String>,
    /// XF index used by the cached numeric result.
    pub format_index: u16,
    /// Cached result type.
    pub cached_type: FormulaCachedType,
    /// Numeric cached value when `cached_type == Numeric`.
    pub number_value: Option<f64>,
    /// Boolean cached value when `cached_type == Boolean`.
    pub bool_value: Option<bool>,
    /// String cached value (`StringRecord` or `#VALUE!` for errors).
    pub string_value: Option<String>,
    /// Whether the string result is pending a following `StringRecord`.
    pub pending_string: bool,
}

