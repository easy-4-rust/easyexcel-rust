//! 模板填充的累积状态结构。
//!
//! 对应 Java：内部辅助类型（填充累积状态）。

use crate::core::CellValue;

use crate::{FillConfig, FillWrapper, TemplateData, TemplateSheet};

include!("sheet_fill_state/pending_collection_fill.rs");

include!("sheet_fill_state/pending_sheet_fill.rs");

#[derive(Debug)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) struct ResolvedSheetFill {
    pub(crate) worksheet: String,
    pub(crate) scalar: TemplateData,
    pub(crate) collections: Vec<PendingCollectionFill>,
    pub(crate) appended_rows: Vec<Vec<CellValue>>,
}
