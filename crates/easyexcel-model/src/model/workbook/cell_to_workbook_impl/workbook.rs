/// 对应 Java：无直接对应对象；Rust 架构扩展。 The top-level spreadsheet: a collection of [`Sheet`]s plus shared state.
#[derive(Debug, Clone)]
pub struct Workbook {
    pub sheets: Vec<Sheet>,
    pub styles: StyleTable,
    pub defined_names: Vec<DefinedName>,
    pub date_system: DateSystem,
    pub metadata: Metadata,
    /// Workbook-level opaque parts (themes, pivot caches, VBA project, …).
    pub opaque: Vec<OpaquePart>,
    /// Active sheet index for the TUI.
    pub active_sheet: usize,
}

impl Default for Workbook {
    fn default() -> Self {
        Workbook::new()
    }
}

