//! Mirrors Java `com.alibaba.excel.annotation.ExcelIgnoreUnannotated`.
//!
//! In Rust, `#[derive(ExcelRow)]` with `#[excel(...)]` attributes
//! replaces Java runtime annotation processing. This module exists
//! for 1:1 Java file parity.

/// Marker type mirroring Java `@ExcelIgnoreUnannotated`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ExcelIgnoreUnannotated;

impl ExcelIgnoreUnannotated {
    /// Creates the type-level ignore-unannotated marker.
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
        // 对应 Java：@ExcelIgnoreUnannotated 标记
        assert_eq!(ExcelIgnoreUnannotated::new(), ExcelIgnoreUnannotated);
        assert_eq!(ExcelIgnoreUnannotated::default(), ExcelIgnoreUnannotated);
    }
}
