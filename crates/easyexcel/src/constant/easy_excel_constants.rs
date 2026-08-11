//! 对应 Java：`com.alibaba.excel.constant.EasyExcelConstants`.

/// Excel stores numbers with 15 significant digits using HALF_UP.
pub use easyexcel_format::{EXCEL_MATH_CONTEXT, EXCEL_MATH_CONTEXT_PRECISION};

/// Java `EasyExcelConstants` 静态常量门面。
#[derive(Debug, Clone, Copy, Default)]
pub struct EasyExcelConstants;

impl EasyExcelConstants {
    /// Excel 数字有效位数，对应 Java `EXCEL_MATH_CONTEXT` 的 precision。
    pub const EXCEL_MATH_CONTEXT_PRECISION: u32 = EXCEL_MATH_CONTEXT_PRECISION;

    /// 返回 Java `EXCEL_MATH_CONTEXT` 的完整 precision + HALF_UP 载体。
    #[must_use]
    pub fn excel_math_context() -> &'static bigdecimal::Context {
        &EXCEL_MATH_CONTEXT
    }
}
