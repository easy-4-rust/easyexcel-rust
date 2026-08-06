#[derive(Debug)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct PendingSheetFill {
    pub(crate) sheet: TemplateSheet,
    pub(crate) scalar: TemplateData,
    pub(crate) collections: Vec<PendingCollectionFill>,
    pub(crate) appended_rows: Vec<Vec<CellValue>>,
}

impl PendingSheetFill {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub(crate) fn new(sheet: TemplateSheet) -> Self {
        Self {
            sheet,
            scalar: TemplateData::new(),
            collections: Vec::new(),
            appended_rows: Vec::new(),
        }
    }
}

