//! Java `NumberUtils` 与 `DecimalFormat` 兼容实现。

mod builtin_formats;
mod data_formatter;
mod excel_locale;
mod excel_general_number_format;
mod locale_generated;
mod number_utils;

pub use builtin_formats::{
    BUILTIN_FORMATS_ALL_LANGUAGES, BUILTIN_FORMATS_CN, GENERAL, MIN_CUSTOM_DATA_FORMAT_INDEX,
    builtin_format_code, get_builtin_format, switch_builtin_formats,
};
pub use data_formatter::{
    excel_display_number, format_raw_cell_contents, format_with_code, is_scientific_magnitude,
    is_date_format_code, java_compat_date_format_code, java_compat_display,
    java_compat_format_code, java_plain_extreme_format, java_scientific_format,
    resolve_builtin_format_code, SpreadsheetLocale,
};
pub use excel_locale::ExcelLocale;
pub use excel_general_number_format::format_general;
pub use number_utils::{
    EXCEL_MATH_CONTEXT_PRECISION, NonFiniteNumber, NumberFormatError, NumberRoundingMode,
    decimal_integer_requires_text,
    decimal_to_big_int, decimal_to_java_i16, decimal_to_java_i32, decimal_to_java_i64,
    decimal_to_java_i8, excel_date_format_code, finite_decimal_f64, format_decimal,
    format_non_finite, java_f32_string, java_f64_string,
    parse_big_decimal, parse_big_int, parse_byte, parse_decimal, parse_double, parse_float,
    parse_integer, parse_long, parse_short,
};
