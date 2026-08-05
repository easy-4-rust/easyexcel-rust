//! XLSX A1 单元格引用与区域解析。

use easyexcel_io::{Error, Result};

/// XLSX 工作表允许的最大行数。
pub const MAX_XLSX_ROW_NUMBER: u32 = 1_048_576;

/// XLSX 工作表允许的最大列数。
pub const MAX_XLSX_COLUMN_NUMBER: usize = 16_384;

/// 解析 XLSX XML 中的一基行号并返回零基坐标。
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

/// 解析 OOXML 中用于样式、共享字符串等表索引的无符号整数。
///
/// # Errors
///
/// 属性值不是平台可表示的无符号下标时返回 XLSX 格式错误。
pub fn parse_xlsx_index(value: &str, attribute: &str) -> Result<usize> {
    value.parse::<usize>().map_err(|error| {
        Error::Xlsx(format!(
            "invalid XLSX {attribute} index {value:?}: {error}"
        ))
    })
}

/// 解析 A1 单元格引用，返回零基行列坐标。
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
        return Err(Error::Xlsx(format!(
            "invalid cell reference: {reference}"
        )));
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

/// 解析 A1 或 A1:B2 区域，返回零基首尾行列坐标。
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

/// 返回工作表 dimension 引用中的最后一行（零基）。
///
/// # Errors
///
/// dimension 尾部引用无效时返回格式错误。
pub fn dimension_last_row(reference: &str) -> Result<u32> {
    let end = reference
        .rsplit_once(':')
        .map_or(reference, |(_, end)| end);
    parse_a1_cell_reference(end).map(|(row, _)| row)
}
