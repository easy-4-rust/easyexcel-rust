//! Java `com.alibaba.excel.util.PositionUtils` 兼容路径。
//!
//! A1 与 OOXML row 坐标算法由 `easyexcel-utils` 唯一实现。

pub use easyexcel_utils::position_utils::{get_col, get_row, get_row_by_row_tagt};
