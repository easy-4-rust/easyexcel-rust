/// 对应 Java：无直接对应对象；Rust 架构扩展。 Options controlling CSV reading.
#[derive(Debug, Clone)]
pub struct CsvReadOptions {
    /// Field delimiter. `None` triggers auto-detection (`,`, `;`, `\t`, `|`).
    pub delimiter: Option<u8>,
    /// Infer cell types (numbers, ISO dates, booleans) instead of all-text.
    pub infer_types: bool,
    /// Sheet name to use for the imported data.
    pub sheet_name: String,
}

impl Default for CsvReadOptions {
    fn default() -> Self {
        CsvReadOptions {
            delimiter: None,
            infer_types: true,
            sheet_name: "Sheet1".to_string(),
        }
    }
}

