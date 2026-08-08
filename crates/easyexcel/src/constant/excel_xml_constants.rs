//! Java `ExcelXmlConstants` 兼容入口；OOXML 协议常量由 `easyexcel-xlsx` 维护。

pub use easyexcel_xlsx::xlsx::ooxml_constants::{
    ATTRIBUTE_LOCATION, ATTRIBUTE_R, ATTRIBUTE_REF, ATTRIBUTE_RID, ATTRIBUTE_S, ATTRIBUTE_T,
    CELL_FORMULA_TAG, CELL_INLINE_STRING_VALUE_TAG, CELL_RANGE_SPLIT, CELL_TAG, CELL_VALUE_TAG,
    DIMENSION_TAG, HYPERLINK_TAG, MERGE_CELL_TAG, NS2_CELL_FORMULA_TAG,
    NS2_CELL_INLINE_STRING_VALUE_TAG, NS2_CELL_TAG, NS2_CELL_VALUE_TAG, NS2_DIMENSION_TAG,
    NS2_HYPERLINK_TAG, NS2_MERGE_CELL_TAG, NS2_ROW_TAG, ROW_TAG, SHAREDSTRINGS_NS2_RPH_TAG,
    SHAREDSTRINGS_NS2_SI_TAG, SHAREDSTRINGS_NS2_T_TAG, SHAREDSTRINGS_RPH_TAG,
    SHAREDSTRINGS_SI_TAG, SHAREDSTRINGS_T_TAG, SHAREDSTRINGS_X_RPH_TAG,
    SHAREDSTRINGS_X_SI_TAG, SHAREDSTRINGS_X_T_TAG, X_CELL_FORMULA_TAG,
    X_CELL_INLINE_STRING_VALUE_TAG, X_CELL_TAG, X_CELL_VALUE_TAG, X_DIMENSION_TAG,
    X_HYPERLINK_TAG, X_MERGE_CELL_TAG, X_ROW_TAG,
};

/// Java `com.alibaba.excel.constant.ExcelXmlConstants` 的静态常量门面。
#[derive(Debug, Clone, Copy, Default)]
pub struct ExcelXmlConstants;

impl ExcelXmlConstants {
    pub const DIMENSION_TAG: &'static str = DIMENSION_TAG;
    pub const ROW_TAG: &'static str = ROW_TAG;
    pub const CELL_FORMULA_TAG: &'static str = CELL_FORMULA_TAG;
    pub const CELL_VALUE_TAG: &'static str = CELL_VALUE_TAG;
    pub const CELL_INLINE_STRING_VALUE_TAG: &'static str = CELL_INLINE_STRING_VALUE_TAG;
    pub const CELL_TAG: &'static str = CELL_TAG;
    pub const MERGE_CELL_TAG: &'static str = MERGE_CELL_TAG;
    pub const HYPERLINK_TAG: &'static str = HYPERLINK_TAG;
    pub const X_DIMENSION_TAG: &'static str = X_DIMENSION_TAG;
    pub const NS2_DIMENSION_TAG: &'static str = NS2_DIMENSION_TAG;
    pub const X_ROW_TAG: &'static str = X_ROW_TAG;
    pub const NS2_ROW_TAG: &'static str = NS2_ROW_TAG;
    pub const X_CELL_FORMULA_TAG: &'static str = X_CELL_FORMULA_TAG;
    pub const NS2_CELL_FORMULA_TAG: &'static str = NS2_CELL_FORMULA_TAG;
    pub const X_CELL_VALUE_TAG: &'static str = X_CELL_VALUE_TAG;
    pub const NS2_CELL_VALUE_TAG: &'static str = NS2_CELL_VALUE_TAG;
    pub const X_CELL_INLINE_STRING_VALUE_TAG: &'static str = X_CELL_INLINE_STRING_VALUE_TAG;
    pub const NS2_CELL_INLINE_STRING_VALUE_TAG: &'static str = NS2_CELL_INLINE_STRING_VALUE_TAG;
    pub const X_CELL_TAG: &'static str = X_CELL_TAG;
    pub const NS2_CELL_TAG: &'static str = NS2_CELL_TAG;
    pub const X_MERGE_CELL_TAG: &'static str = X_MERGE_CELL_TAG;
    pub const NS2_MERGE_CELL_TAG: &'static str = NS2_MERGE_CELL_TAG;
    pub const X_HYPERLINK_TAG: &'static str = X_HYPERLINK_TAG;
    pub const NS2_HYPERLINK_TAG: &'static str = NS2_HYPERLINK_TAG;
    pub const ATTRIBUTE_S: &'static str = ATTRIBUTE_S;
    pub const ATTRIBUTE_REF: &'static str = ATTRIBUTE_REF;
    pub const ATTRIBUTE_R: &'static str = ATTRIBUTE_R;
    pub const ATTRIBUTE_T: &'static str = ATTRIBUTE_T;
    pub const ATTRIBUTE_LOCATION: &'static str = ATTRIBUTE_LOCATION;
    pub const ATTRIBUTE_RID: &'static str = ATTRIBUTE_RID;
    pub const CELL_RANGE_SPLIT: &'static str = CELL_RANGE_SPLIT;
    pub const SHAREDSTRINGS_T_TAG: &'static str = SHAREDSTRINGS_T_TAG;
    pub const SHAREDSTRINGS_X_T_TAG: &'static str = SHAREDSTRINGS_X_T_TAG;
    pub const SHAREDSTRINGS_NS2_T_TAG: &'static str = SHAREDSTRINGS_NS2_T_TAG;
    pub const SHAREDSTRINGS_SI_TAG: &'static str = SHAREDSTRINGS_SI_TAG;
    pub const SHAREDSTRINGS_X_SI_TAG: &'static str = SHAREDSTRINGS_X_SI_TAG;
    pub const SHAREDSTRINGS_NS2_SI_TAG: &'static str = SHAREDSTRINGS_NS2_SI_TAG;
    pub const SHAREDSTRINGS_RPH_TAG: &'static str = SHAREDSTRINGS_RPH_TAG;
    pub const SHAREDSTRINGS_X_RPH_TAG: &'static str = SHAREDSTRINGS_X_RPH_TAG;
    pub const SHAREDSTRINGS_NS2_RPH_TAG: &'static str = SHAREDSTRINGS_NS2_RPH_TAG;
}
