//! Java `StringUtils` 门面重导出。

pub use easyexcel_utils::string_utils::{
    EMPTY, SPACE, equals, equals_with_optional_java_trim, is_blank, is_empty, is_not_blank,
    is_numeric, java_trim, maybe_trim, region_matches, utf8_byte_len_u16,
};
