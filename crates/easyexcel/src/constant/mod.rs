//! 对应 Java：`com.alibaba.excel.constant.*`.

pub mod builtin_formats;
pub mod easy_excel_constants;
pub mod excel_xml_constants;
pub mod order_constant;

pub use builtin_formats::{
    BUILTIN_FORMATS_ALL_LANGUAGES, BUILTIN_FORMATS_CN, GENERAL, MIN_CUSTOM_DATA_FORMAT_INDEX,
    builtin_format_code, get_builtin_format, switch_builtin_formats,
};
pub use easy_excel_constants::EXCEL_MATH_CONTEXT_PRECISION;
pub use excel_xml_constants::{
    ATTRIBUTE_LOCATION, ATTRIBUTE_R, ATTRIBUTE_REF, ATTRIBUTE_RID, ATTRIBUTE_S, ATTRIBUTE_T,
    CELL_FORMULA_TAG, CELL_INLINE_STRING_VALUE_TAG, CELL_RANGE_SPLIT, CELL_TAG, CELL_VALUE_TAG,
    DIMENSION_TAG, HYPERLINK_TAG, MERGE_CELL_TAG, ROW_TAG, SHAREDSTRINGS_RPH_TAG,
    SHAREDSTRINGS_SI_TAG, SHAREDSTRINGS_T_TAG,
};
pub use order_constant::{
    ANNOTATION_DEFINE_STYLE, DEFAULT_DEFINE_STYLE, DEFAULT_ORDER, DEFINE_STYLE, FILL_STYLE,
};
