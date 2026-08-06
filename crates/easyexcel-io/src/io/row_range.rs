//! 与格式无关的物理行范围校验。

use crate::{Error, Result};

/// 校验可选的零基闭区间行范围。
///
/// 仅同时给出起止行且起始行大于结束行时返回错误。
pub fn validate_row_range(start_row: Option<u32>, end_row: Option<u32>) -> Result<()> {
    if let (Some(start), Some(end)) = (start_row, end_row)
        && start > end
    {
        return Err(Error::Other(format!(
            "read row range start {start} exceeds end {end}"
        )));
    }
    Ok(())
}

/// 判断物理行是否应进入上层读取管线。
///
/// 表头行始终保留；数据行再按可选的零基闭区间筛选。调用方应先通过
/// [`validate_row_range`] 校验起止范围。
#[must_use]
pub fn row_is_selected(
    row_index: u32,
    head_row_number: u32,
    start_row: Option<u32>,
    end_row: Option<u32>,
) -> bool {
    row_index < head_row_number
        || (start_row.is_none_or(|start| row_index >= start)
            && end_row.is_none_or(|end| row_index <= end))
}

#[cfg(test)]
mod tests {
    use super::row_is_selected;

    #[test]
    fn headers_are_kept_and_data_range_is_closed() {
        assert!(row_is_selected(0, 2, Some(4), Some(6)));
        assert!(row_is_selected(1, 2, Some(4), Some(6)));
        assert!(!row_is_selected(2, 2, Some(4), Some(6)));
        assert!(row_is_selected(4, 2, Some(4), Some(6)));
        assert!(row_is_selected(6, 2, Some(4), Some(6)));
        assert!(!row_is_selected(7, 2, Some(4), Some(6)));
    }

    #[test]
    fn absent_bounds_select_all_data_rows() {
        assert!(row_is_selected(42, 0, None, None));
    }
}
