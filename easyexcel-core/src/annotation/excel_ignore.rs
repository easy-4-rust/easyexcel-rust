//! 对应 Java：`com.alibaba.excel.annotation.ExcelIgnore`.
//!
//! In Rust, `#[derive(ExcelRow)]` with `#[excel(...)]` attributes
//! replaces Java runtime annotation processing. This module exists
//! for 1:1 Java file parity.

/// Marker type mirroring Java `@ExcelIgnore`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExcelIgnore;

impl ExcelIgnore {
    /// Creates the field-level ignore marker.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn new_creates_marker() {
        // 对应 Java：@ExcelIgnore 标记
        assert_eq!(ExcelIgnore::new(), ExcelIgnore);
        assert_eq!(ExcelIgnore, ExcelIgnore);
    }
}
