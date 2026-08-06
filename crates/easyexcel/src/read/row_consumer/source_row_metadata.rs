/// 对应 Java：无直接对应对象；Rust 架构扩展。 Per-row metadata collected while materializing cells before dispatch.
#[derive(Default)]
pub(crate) struct SourceRowMetadata {
    pub(crate) formulas: HashMap<usize, FormulaData>,
    pub(crate) display_values: HashMap<usize, String>,
    pub(crate) decimal_values: HashMap<usize, bigdecimal::BigDecimal>,
    pub(crate) present_columns: HashSet<usize>,
}

