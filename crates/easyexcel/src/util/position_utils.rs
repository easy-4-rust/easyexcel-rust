//! Java `PositionUtils` 兼容入口。

/// 读取 OOXML `<row r="…">` 的一基行号。
#[must_use]
pub fn get_row_by_row_tagt(row_tag: &str) -> u32 {
    easyexcel_xlsx::parse_xlsx_row_number(row_tag)
        .map(|row| row + 1)
        .unwrap_or(0)
}

/// 从 A1 引用读取零基行号。
#[must_use]
pub fn get_row(cell_ref: &str) -> u32 {
    easyexcel_model::addr::row_from_a1(cell_ref)
}

/// 从 A1 引用读取零基列号。
#[must_use]
pub fn get_col(cell_ref: &str) -> u32 {
    easyexcel_model::addr::column_from_a1(cell_ref)
}
