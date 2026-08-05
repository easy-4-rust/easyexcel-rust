//! 数字与电子表格显示格式引擎的统一公开入口。

pub use easyexcel_format::format;
pub use easyexcel_format::{
    BUILTIN_FORMATS_ALL_LANGUAGES, BUILTIN_FORMATS_CN, EXCEL_MATH_CONTEXT_PRECISION, ExcelLocale,
    GENERAL, MIN_CUSTOM_DATA_FORMAT_INDEX, NonFiniteNumber, NumberFormatError, NumberRoundingMode,
    SpreadsheetLocale, builtin_format_code, decimal_integer_requires_text, decimal_to_big_int,
    decimal_to_java_i8, decimal_to_java_i16, decimal_to_java_i32, decimal_to_java_i64,
    excel_date_format_code, excel_display_number, finite_decimal_f64, format_decimal,
    format_general, format_non_finite, format_raw_cell_contents, format_with_code,
    get_builtin_format, is_date_format_code, is_scientific_magnitude, java_compat_date_format_code,
    java_compat_display, java_compat_format_code, java_f32_string, java_f64_string,
    java_plain_extreme_format, java_scientific_format, parse_big_decimal, parse_big_int, parse_byte,
    parse_decimal, parse_double, parse_float, parse_integer, parse_long, parse_short,
    resolve_builtin_format_code, switch_builtin_formats,
};
