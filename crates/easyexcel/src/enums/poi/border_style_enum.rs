//! 对应 Java：`com.alibaba.excel.enums.poi.BorderStyleEnum`.
//!
//! POI-specific enum wrapper. Rust already has `ExcelBorderStyle` in core;
//! this module re-exports it under the Java POI name for 1:1 parity.

/// Re-export of [`crate::ExcelBorderStyle`] matching Java's POI enum name.
/// 对应 Java：com.alibaba.excel.enums.poi.BorderStyleEnum。
pub type BorderStyleEnum = crate::ExcelBorderStyle;
