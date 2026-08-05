//! CSV 物理行列索引的有界转换。

use easyexcel_io::{Error, Result};

/// 将平台相关的行下标转换为 CSV 工作簿使用的 `u32` 行号。
///
/// # Errors
///
/// 下标超过 `u32` 可表示范围时返回 CSV 格式错误。
pub fn checked_row_index(index: usize) -> Result<u32> {
    u32::try_from(index).map_err(|_| Error::Csv("CSV row index exceeds u32".to_owned()))
}

/// 将平台相关的列下标转换为 CSV 工作簿使用的 `u32` 列号。
///
/// # Errors
///
/// 下标超过 `u32` 可表示范围时返回 CSV 格式错误。
pub fn checked_column_index(index: usize) -> Result<u32> {
    u32::try_from(index).map_err(|_| Error::Csv("CSV column index exceeds u32".to_owned()))
}
