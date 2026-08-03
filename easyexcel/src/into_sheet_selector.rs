//! `.sheet(...)` 入参的统一转换 trait。
//!
//! 对应 Java：`com.alibaba.excel.read.builder.ExcelReaderBuilder.sheet(...)` 接受的
//! `Integer` / `String` 形参，由 Rust 在 facade 层做工厂式归一。

use crate::reader::SheetSelector;

/// Input accepted by `.sheet(...)`.
pub trait IntoSheetSelector {
    /// Converts to internal sheet selection.
    fn into_sheet_selector(self) -> SheetSelector;
}

impl IntoSheetSelector for usize {
    fn into_sheet_selector(self) -> SheetSelector {
        SheetSelector::Index(self)
    }
}

impl IntoSheetSelector for &str {
    fn into_sheet_selector(self) -> SheetSelector {
        SheetSelector::Name(self.to_owned())
    }
}

impl IntoSheetSelector for String {
    fn into_sheet_selector(self) -> SheetSelector {
        SheetSelector::Name(self)
    }
}
