//! Java `SheetUtils` 门面重导出。

pub use easyexcel_utils::sheet_utils::{MAX_COLUMN_WIDTH_CHARS, match_sheet};

/// Java `SheetUtils.match` 命名兼容；Rust 关键字使用 raw identifier。
#[must_use]
pub fn r#match(sheet_name: &str, requested_sheet_name: &str) -> bool {
    match_sheet(sheet_name, requested_sheet_name)
}
