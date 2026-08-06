#[derive(Clone)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct StatefulSheetState {
    pub(crate) schema: &'static [ExcelColumn],
    pub(crate) metadata: ExcelWriteMetadata,
    pub(crate) options: WriteOptions,
    pub(crate) next_row: u32,
    pub(crate) next_data_index: usize,
}

