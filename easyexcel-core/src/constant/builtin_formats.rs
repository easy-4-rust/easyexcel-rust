//! 对应 Java：`com.alibaba.excel.constant.BuiltinFormats`.
//!
//! Java defines three locale-dependent arrays (`BUILTIN_FORMATS_ALL_LANGUAGES`,
//! `BUILTIN_FORMATS_CN`, `BUILTIN_FORMATS_US`) plus lookup helpers. The Rust
//! port delegates the actual format-code resolution to the `ssfmt` crate;
//! this module mirrors the constant arrays for 1:1 API parity.

/// The minimum custom format index. (Java `MIN_CUSTOM_DATA_FORMAT_INDEX`)
pub const MIN_CUSTOM_DATA_FORMAT_INDEX: u16 = 82;

/// The General format index. (Java `GENERAL`)
pub const GENERAL: u16 = 0;

/// Returns the built-in format string for the given index and locale.
/// (Java `getBuiltinFormat(Short, String, Locale)`)
///
/// Lookup order matches Java: `ALL_LANGUAGES` first, then CN locale table,
/// finally `default_format` / `"General"`.
#[must_use]
pub fn get_builtin_format(index: u16, default_format: &str) -> &'static str {
    if let Some(code) = builtin_format_code(index) {
        return code;
    }
    if default_format.is_empty() {
        "General"
    } else {
        // Callers that need the exact non-static default must branch themselves;
        // this API mirrors Java's static builtin tables.
        "General"
    }
}

/// Resolves a builtin format code the same way `EasyExcel` STRING display does.
#[must_use]
pub fn builtin_format_code(index: u16) -> Option<&'static str> {
    BUILTIN_FORMATS_ALL_LANGUAGES
        .get(index as usize)
        .copied()
        .flatten()
        .or_else(|| BUILTIN_FORMATS_CN.get(index as usize).copied().flatten())
}

/// Returns the built-in format array. (Java `switchBuiltinFormats(Locale)`)
#[must_use]
pub fn switch_builtin_formats() -> &'static [Option<&'static str>] {
    &BUILTIN_FORMATS_ALL_LANGUAGES
}

/// The "all languages" built-in format table. (Java
/// `BUILTIN_FORMATS_ALL_LANGUAGES`)
pub static BUILTIN_FORMATS_ALL_LANGUAGES: [Option<&str>; 50] = [
    Some("General"),                                // 0
    Some("0"),                                      // 1
    Some("0.00"),                                   // 2
    Some("#,##0"),                                  // 3
    Some("#,##0.00"),                               // 4
    Some("\"￥\"#,##0_);(\"￥\"#,##0)"),            // 5
    Some("\"￥\"#,##0_);[Red](\"￥\"#,##0)"),       // 6
    Some("\"￥\"#,##0.00_);(\"￥\"#,##0.00)"),      // 7
    Some("\"￥\"#,##0.00_);[Red](\"￥\"#,##0.00)"), // 8
    Some("0%"),                                     // 9
    Some("0.00%"),                                  // 10
    Some("0.00E+00"),                               // 11
    Some("# ?/?"),                                  // 12
    Some("# ??/??"),                                // 13
    Some("yyyy/m/d"),                               // 14
    Some("d-mmm-yy"),                               // 15
    Some("d-mmm"),                                  // 16
    Some("mmm-yy"),                                 // 17
    Some("h:mm AM/PM"),                             // 18
    Some("h:mm:ss AM/PM"),                          // 19
    Some("h:mm"),                                   // 20
    Some("h:mm:ss"),                                // 21
    Some("yyyy-m-d h:mm"),                          // 22
    None,
    None,
    None,
    None, // 23-26
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,                                                                        // 27-36
    Some("#,##0_);(#,##0)"),                                                     // 37
    Some("#,##0_);[Red](#,##0)"),                                                // 38
    Some("#,##0.00_);(#,##0.00)"),                                               // 39
    Some("#,##0.00_);[Red](#,##0.00)"),                                          // 40
    Some("_(* #,##0_);_(* (#,##0);_(* \"-\"_);_(@_)"),                           // 41
    Some("_(\"￥\"* #,##0_);_(\"￥\"* (#,##0);_(\"￥\"* \"-\"_);_(@_)"),         // 42
    Some("_(* #,##0.00_);_(* (#,##0.00);_(* \"-\"??_);_(@_)"),                   // 43
    Some("_(\"￥\"* #,##0.00_);_(\"￥\"* (#,##0.00);_(\"￥\"* \"-\"??_);_(@_)"), // 44
    Some("mm:ss"),                                                               // 45
    Some("[h]:mm:ss"),                                                           // 46
    Some("mm:ss.0"),                                                             // 47
    Some("##0.0E+0"),                                                            // 48
    Some("@"),                                                                   // 49
];

/// Locale-CN built-in formats used when `ALL_LANGUAGES` has no entry.
/// (Java `BUILTIN_FORMATS_CN` — date/time slots 27–36 / 50–58)
pub static BUILTIN_FORMATS_CN: [Option<&str>; 59] = [
    Some("General"),                                // 0
    Some("0"),                                      // 1
    Some("0.00"),                                   // 2
    Some("#,##0"),                                  // 3
    Some("#,##0.00"),                               // 4
    Some("\"￥\"#,##0_);(\"￥\"#,##0)"),            // 5
    Some("\"￥\"#,##0_);[Red](\"￥\"#,##0)"),       // 6
    Some("\"￥\"#,##0.00_);(\"￥\"#,##0.00)"),      // 7
    Some("\"￥\"#,##0.00_);[Red](\"￥\"#,##0.00)"), // 8
    Some("0%"),                                     // 9
    Some("0.00%"),                                  // 10
    Some("0.00E+00"),                               // 11
    Some("# ?/?"),                                  // 12
    Some("# ??/??"),                                // 13
    Some("yyyy/m/d"),                               // 14
    Some("d-mmm-yy"),                               // 15
    Some("d-mmm"),                                  // 16
    Some("mmm-yy"),                                 // 17
    Some("h:mm AM/PM"),                             // 18
    Some("h:mm:ss AM/PM"),                          // 19
    Some("h:mm"),                                   // 20
    Some("h:mm:ss"),                                // 21
    Some("yyyy-m-d h:mm"),                          // 22
    None,
    None,
    None,
    None,                                                                        // 23-26
    Some("yyyy\"年\"m\"月\""),                                                   // 27
    Some("m\"月\"d\"日\""),                                                      // 28
    Some("m\"月\"d\"日\""),                                                      // 29
    Some("m-d-yy"),                                                              // 30
    Some("yyyy\"年\"m\"月\"d\"日\""),                                            // 31
    Some("h\"时\"mm\"分\""),                                                     // 32
    Some("h\"时\"mm\"分\"ss\"秒\""),                                             // 33
    Some("上午/下午h\"时\"mm\"分\""),                                            // 34
    Some("上午/下午h\"时\"mm\"分\"ss\"秒\""),                                    // 35
    Some("yyyy\"年\"m\"月\""),                                                   // 36
    Some("#,##0_);(#,##0)"),                                                     // 37
    Some("#,##0_);[Red](#,##0)"),                                                // 38
    Some("#,##0.00_);(#,##0.00)"),                                               // 39
    Some("#,##0.00_);[Red](#,##0.00)"),                                          // 40
    Some("_(* #,##0_);_(* (#,##0);_(* \"-\"_);_(@_)"),                           // 41
    Some("_(\"￥\"* #,##0_);_(\"￥\"* (#,##0);_(\"￥\"* \"-\"_);_(@_)"),         // 42
    Some("_(* #,##0.00_);_(* (#,##0.00);_(* \"-\"??_);_(@_)"),                   // 43
    Some("_(\"￥\"* #,##0.00_);_(\"￥\"* (#,##0.00);_(\"￥\"* \"-\"??_);_(@_)"), // 44
    Some("mm:ss"),                                                               // 45
    Some("[h]:mm:ss"),                                                           // 46
    Some("mm:ss.0"),                                                             // 47
    Some("##0.0E+0"),                                                            // 48
    Some("@"),                                                                   // 49
    Some("yyyy\"年\"m\"月\""),                                                   // 50
    Some("m\"月\"d\"日\""),                                                      // 51
    Some("yyyy\"年\"m\"月\""),                                                   // 52
    Some("m\"月\"d\"日\""),                                                      // 53
    Some("m\"月\"d\"日\""),                                                      // 54
    Some("上午/下午h\"时\"mm\"分\""),                                            // 55
    Some("上午/下午h\"时\"mm\"分\"ss\"秒\""),                                    // 56
    Some("yyyy\"年\"m\"月\""),                                                   // 57
    Some("m\"月\"d\"日\""),                                                      // 58
];

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn get_builtin_format_resolves_tables() {
        // 对应 Java：getBuiltinFormat 查表顺序
        assert_eq!(get_builtin_format(0, ""), "General");
        assert_eq!(get_builtin_format(14, ""), "yyyy/m/d");
        assert_eq!(get_builtin_format(49, ""), "@");
        // 超出表范围回退 General
        assert_eq!(get_builtin_format(99, ""), "General");
        assert_eq!(get_builtin_format(99, "0.00"), "General");
    }

    #[test]
    fn builtin_format_code_falls_back_to_cn_table() {
        // 对应 Java：ALL_LANGUAGES 为空时回退 CN 表
        assert_eq!(builtin_format_code(0), Some("General"));
        assert_eq!(builtin_format_code(14), Some("yyyy/m/d"));
        // 索引 27 在 ALL_LANGUAGES 为空，回退 CN 表
        assert_eq!(builtin_format_code(27), Some("yyyy\"年\"m\"月\""));
        assert_eq!(builtin_format_code(99), None);
        let all = switch_builtin_formats();
        assert_eq!(all.len(), 50);
        assert_eq!(all[0], Some("General"));
    }
}
