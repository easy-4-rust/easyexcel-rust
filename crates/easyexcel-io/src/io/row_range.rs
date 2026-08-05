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
