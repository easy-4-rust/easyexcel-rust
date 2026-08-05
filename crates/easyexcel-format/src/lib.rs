//! 数字、日期和电子表格显示格式的可复用算法层。

pub mod format;

pub use format::{
    BUILTIN_FORMATS_ALL_LANGUAGES, BUILTIN_FORMATS_CN, GENERAL, MIN_CUSTOM_DATA_FORMAT_INDEX,
    ExcelLocale, NonFiniteNumber, NumberFormatError, NumberRoundingMode, builtin_format_code,
    decimal_integer_requires_text, decimal_to_big_int, decimal_to_java_i16, decimal_to_java_i32,
    decimal_to_java_i64, decimal_to_java_i8, excel_date_format_code, excel_display_number,
    finite_decimal_f64, format_decimal, format_general, format_non_finite, java_f32_string,
    java_f64_string,
    format_raw_cell_contents, format_with_code, get_builtin_format, is_scientific_magnitude,
    is_date_format_code, java_compat_date_format_code, java_compat_display,
    java_compat_format_code, java_plain_extreme_format, java_scientific_format,
    parse_big_decimal, parse_big_int, parse_byte, parse_decimal, parse_double, parse_float,
    parse_integer, parse_long, parse_short, resolve_builtin_format_code, switch_builtin_formats,
    SpreadsheetLocale,
};
