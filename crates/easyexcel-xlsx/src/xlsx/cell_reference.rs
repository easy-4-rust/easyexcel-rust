//! XLSX A1 单元格引用与区域解析。

use easyexcel_io::{Error, Result};

/// XLSX 工作表允许的最大行数。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MAX_XLSX_ROW_NUMBER: u32 = 1_048_576;

/// XLSX 工作表允许的最大列数。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const MAX_XLSX_COLUMN_NUMBER: usize = 16_384;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 XLSX XML 中的一基行号并返回零基坐标。
///
/// # Errors
///
/// 行号不是正整数或超过 XLSX 行上限时返回格式错误。
pub fn parse_xlsx_row_number(value: &str) -> Result<u32> {
    let one_based = value
        .parse::<u32>()
        .map_err(|error| Error::Xlsx(error.to_string()))?;
    if !(1..=MAX_XLSX_ROW_NUMBER).contains(&one_based) {
        return Err(Error::Xlsx(format!(
            "row index exceeds XLSX limits: {value}"
        )));
    }
    Ok(one_based - 1)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 OOXML 中用于样式、共享字符串等表索引的无符号整数。
///
/// # Errors
///
/// 属性值不是平台可表示的无符号下标时返回 XLSX 格式错误。
pub fn parse_xlsx_index(value: &str, attribute: &str) -> Result<usize> {
    value
        .parse::<usize>()
        .map_err(|error| Error::Xlsx(format!("invalid XLSX {attribute} index {value:?}: {error}")))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 A1 单元格引用，返回零基行列坐标。
///
/// 支持绝对引用中的 `$` 前缀，并校验 XLSX 行列上限。
///
/// # Errors
///
/// 引用语法无效或超出 XLSX 行列上限时返回格式错误。
pub fn parse_a1_cell_reference(reference: &str) -> Result<(u32, usize)> {
    let reference = reference.strip_prefix('$').unwrap_or(reference);
    let column_end = reference
        .find(|character: char| !character.is_ascii_alphabetic())
        .unwrap_or(reference.len());
    let (column, row) = reference.split_at(column_end);
    let row = row.strip_prefix('$').unwrap_or(row);
    if column.is_empty() || row.is_empty() || !row.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(Error::Xlsx(format!("invalid cell reference: {reference}")));
    }

    let mut one_based_column = 0_usize;
    for letter in column.bytes() {
        one_based_column = one_based_column
            .checked_mul(26)
            .and_then(|value| {
                value.checked_add(usize::from(letter.to_ascii_uppercase() - b'A' + 1))
            })
            .ok_or_else(|| Error::Xlsx(format!("invalid cell reference: {reference}")))?;
    }
    if !(1..=MAX_XLSX_COLUMN_NUMBER).contains(&one_based_column) {
        return Err(Error::Xlsx(format!(
            "column index exceeds XLSX limits: {reference}"
        )));
    }

    Ok((parse_xlsx_row_number(row)?, one_based_column - 1))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 A1 或 A1:B2 区域，返回零基首尾行列坐标。
///
/// # Errors
///
/// 任一单元格引用无效、超限，或区域首坐标大于尾坐标时返回格式错误。
pub fn parse_a1_cell_range(reference: &str) -> Result<(u32, u32, usize, usize)> {
    let (first, last) = reference
        .split_once(':')
        .map_or((reference, reference), |range| range);
    let (first_row, first_column) = parse_a1_cell_reference(first)?;
    let (last_row, last_column) = parse_a1_cell_reference(last)?;
    if first_row > last_row || first_column > last_column {
        return Err(Error::Xlsx(format!(
            "invalid cell range ordering: {reference}"
        )));
    }
    Ok((first_row, last_row, first_column, last_column))
}

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 返回工作表 dimension 引用中的最后一行（零基）。
    ///
    /// # Errors
    ///
    /// dimension 尾部引用无效时返回格式错误。
    pub fn dimension_last_row(reference: &str) -> Result<u32> {
        let end = reference.rsplit_once(':').map_or(reference, |(_, end)| end);
        parse_a1_cell_reference(end).map(|(row, _)| row)
    }

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_xlsx_row_number 覆盖 ─────────────────────────────────────────

    #[test]
    fn parse_xlsx_row_number_valid() {
        assert_eq!(parse_xlsx_row_number("1").unwrap(), 0);
        assert_eq!(parse_xlsx_row_number("100").unwrap(), 99);
        assert_eq!(parse_xlsx_row_number("1048576").unwrap(), 1_048_575);
    }

    #[test]
    fn parse_xlsx_row_number_zero_rejected() {
        assert!(parse_xlsx_row_number("0").is_err());
    }

    #[test]
    fn parse_xlsx_row_number_exceeds_max() {
        assert!(parse_xlsx_row_number("1048577").is_err());
    }

    #[test]
    fn parse_xlsx_row_number_invalid_format() {
        assert!(parse_xlsx_row_number("abc").is_err());
    }

    // ── parse_xlsx_index 覆盖 ──────────────────────────────────────────────

    #[test]
    fn parse_xlsx_index_valid() {
        assert_eq!(parse_xlsx_index("0", "test").unwrap(), 0);
        assert_eq!(parse_xlsx_index("42", "test").unwrap(), 42);
    }

    #[test]
    fn parse_xlsx_index_invalid() {
        assert!(parse_xlsx_index("abc", "style").is_err());
    }

    // ── parse_a1_cell_reference 覆盖 ───────────────────────────────────────

    #[test]
    fn parse_a1_simple_reference() {
        let (row, col) = parse_a1_cell_reference("A1").unwrap();
        assert_eq!(row, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn parse_a1_absolute_reference() {
        let (row, col) = parse_a1_cell_reference("$B$5").unwrap();
        assert_eq!(row, 4);
        assert_eq!(col, 1);
    }

    #[test]
    fn parse_a1_mixed_reference() {
        let (row, col) = parse_a1_cell_reference("$C10").unwrap();
        assert_eq!(row, 9);
        assert_eq!(col, 2);
    }

    #[test]
    fn parse_a1_multi_letter_column() {
        let (row, col) = parse_a1_cell_reference("AA1").unwrap();
        assert_eq!(row, 0);
        assert_eq!(col, 26);
    }

    #[test]
    fn parse_a1_max_column() {
        let (row, col) = parse_a1_cell_reference("XFD1").unwrap();
        assert_eq!(row, 0);
        assert_eq!(col, 16_383);
    }

    #[test]
    fn parse_a1_exceeds_max_column() {
        assert!(parse_a1_cell_reference("XFE1").is_err());
    }

    #[test]
    fn parse_a1_empty_column() {
        assert!(parse_a1_cell_reference("1").is_err());
    }

    #[test]
    fn parse_a1_empty_row() {
        assert!(parse_a1_cell_reference("A").is_err());
    }

    #[test]
    fn parse_a1_invalid_characters() {
        assert!(parse_a1_cell_reference("A1B").is_err());
    }

    // ── parse_a1_cell_range 覆盖 ───────────────────────────────────────────

    #[test]
    fn parse_a1_cell_range_valid() {
        let (fr, lr, fc, lc) = parse_a1_cell_range("A1:C3").unwrap();
        assert_eq!((fr, lr, fc, lc), (0, 2, 0, 2));
    }

    #[test]
    fn parse_a1_cell_range_single_cell() {
        let (fr, lr, fc, lc) = parse_a1_cell_range("B2").unwrap();
        assert_eq!((fr, lr, fc, lc), (1, 1, 1, 1));
    }

    #[test]
    fn parse_a1_cell_range_invalid_ordering() {
        // first > last
        assert!(parse_a1_cell_range("C3:A1").is_err());
    }

    #[test]
    fn parse_a1_cell_range_column_ordering() {
        assert!(parse_a1_cell_range("C1:A1").is_err());
    }

    // ── dimension_last_row 覆盖 ────────────────────────────────────────────

    #[test]
    fn dimension_last_row_range() {
        assert_eq!(dimension_last_row("A1:C10").unwrap(), 9);
    }

    #[test]
    fn dimension_last_row_single_cell() {
        assert_eq!(dimension_last_row("B5").unwrap(), 4);
    }
}
