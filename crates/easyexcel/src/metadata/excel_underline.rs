//! 对应 Java：`com.alibaba.excel.enums.poi.FontUnderline` styling subset.

/// Font underline style used by annotation-driven font styles.
///
/// Java uses POI `FontUnderline` codes (0..=5); Rust drops them. Variant names
/// follow the POI enum names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// 对应 Java：com.alibaba.excel.enums.poi.FontUnderline。
pub enum ExcelUnderline {
    /// No underline.
    None,
    /// Single underline.
    Single,
    /// Double underline.
    Double,
    /// Single accounting underline.
    SingleAccounting,
    /// Double accounting underline.
    DoubleAccounting,
}
