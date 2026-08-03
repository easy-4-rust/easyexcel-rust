//! 模板填充的累积状态结构。
//!
//! 对应 Java：内部辅助类型（填充累积状态）。

use crate::core::CellValue;

use crate::{FillConfig, FillWrapper, TemplateData, TemplateSheet};

#[derive(Debug, Clone)]
pub(crate) struct PendingCollectionFill {
    pub(crate) wrapper: FillWrapper,
    pub(crate) config: FillConfig,
    pub(crate) order: usize,
}

#[derive(Debug)]
pub(crate) struct PendingSheetFill {
    pub(crate) sheet: TemplateSheet,
    pub(crate) scalar: TemplateData,
    pub(crate) collections: Vec<PendingCollectionFill>,
    pub(crate) appended_rows: Vec<Vec<CellValue>>,
}

#[derive(Debug)]
pub(crate) struct ResolvedSheetFill {
    pub(crate) worksheet: String,
    pub(crate) scalar: TemplateData,
    pub(crate) collections: Vec<PendingCollectionFill>,
    pub(crate) appended_rows: Vec<Vec<CellValue>>,
}

impl PendingSheetFill {
    pub(crate) fn new(sheet: TemplateSheet) -> Self {
        Self {
            sheet,
            scalar: TemplateData::new(),
            collections: Vec::new(),
            appended_rows: Vec::new(),
        }
    }
}
