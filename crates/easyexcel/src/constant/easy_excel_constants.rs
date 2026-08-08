//! 对应 Java：`com.alibaba.excel.constant.EasyExcelConstants`.

/// Excel stores numbers with 15 significant digits. (Java `EXCEL_MATH_CONTEXT`)
pub use easyexcel_format::EXCEL_MATH_CONTEXT_PRECISION;

/// Java `EasyExcelConstants` 静态常量门面。
#[derive(Debug, Clone, Copy, Default)]
pub struct EasyExcelConstants;

impl EasyExcelConstants {
    /// Excel 数字有效位数，对应 Java `EXCEL_MATH_CONTEXT` 的 precision。
    pub const EXCEL_MATH_CONTEXT_PRECISION: u32 = EXCEL_MATH_CONTEXT_PRECISION;
}
