//! 对应 Java：`com.alibaba.excel.constant.BuiltinFormats`.
//!
//! Java defines three locale-dependent arrays (`BUILTIN_FORMATS_ALL_LANGUAGES`,
//! `BUILTIN_FORMATS_CN`, `BUILTIN_FORMATS_US`) plus lookup helpers. The Rust
//! port delegates the actual format-code resolution to the `ssfmt` crate;
//! this module mirrors the constant arrays for 1:1 API parity.

use std::collections::HashMap;
use std::sync::LazyLock;

use super::ExcelLocale;

const RESERVED: &str = "reserved-";

/// The minimum custom format index. (Java `MIN_CUSTOM_DATA_FORMAT_INDEX`)
/// 对应 Java：com.alibaba.excel.constant.BuiltinFormats。
pub const MIN_CUSTOM_DATA_FORMAT_INDEX: u16 = 82;

/// The General format index. (Java `GENERAL`)
/// 对应 Java：com.alibaba.excel.constant.BuiltinFormats。
pub const GENERAL: u16 = 0;

/// 对应 Java：com.alibaba.excel.constant.BuiltinFormats。 Returns the built-in format string for the given index and locale.
/// (Java `getBuiltinFormat(Short, String, Locale)`)
///
/// Lookup order matches Java: `ALL_LANGUAGES` first, then CN locale table,
/// finally `default_format` / `"General"`.
#[must_use]
pub fn get_builtin_format(index: u16, default_format: &str) -> &str {
    get_builtin_format_for_locale(Some(index), Some(default_format), None)
        .unwrap_or(default_format)
}

/// 按 Java 的查找顺序解析内建格式，并保留调用者提供的默认格式。
#[must_use]
pub fn get_builtin_format_for_locale<'a>(
    index: Option<u16>,
    default_format: Option<&'a str>,
    locale: Option<&ExcelLocale>,
) -> Option<&'a str> {
    let index = index?;
    if index == 0 {
        return default_format;
    }
    if let Some(format) = BUILTIN_FORMATS_ALL_LANGUAGES
        .get(index as usize)
        .copied()
        .flatten()
    {
        return Some(format);
    }
    if default_format.is_some_and(|value| !value.is_empty() && !value.starts_with(RESERVED)) {
        return default_format;
    }
    switch_builtin_formats_for_locale(locale)
        .get(index as usize)
        .copied()
        .flatten()
        .or(default_format)
}

/// 对应 Java：com.alibaba.excel.constant.BuiltinFormats。 Resolves a builtin format code the same way `EasyExcel` STRING display does.
#[must_use]
pub fn builtin_format_code(index: u16) -> Option<&'static str> {
    BUILTIN_FORMATS_ALL_LANGUAGES
        .get(index as usize)
        .copied()
        .flatten()
        .or_else(|| BUILTIN_FORMATS_CN.get(index as usize).copied().flatten())
}

/// 对应 Java：com.alibaba.excel.constant.BuiltinFormats。 Returns the built-in format array. (Java `switchBuiltinFormats(Locale)`)
#[must_use]
pub fn switch_builtin_formats() -> &'static [Option<&'static str>] {
    &BUILTIN_FORMATS_CN
}

/// 根据国家代码选择 Java 对应的 US/CN 内建格式表。
#[must_use]
pub fn switch_builtin_formats_for_locale(
    locale: Option<&ExcelLocale>,
) -> &'static [Option<&'static str>] {
    if locale.is_some_and(|value| {
        let tag = value.language_tag();
        tag.eq_ignore_ascii_case("US")
            || tag.ends_with("_US")
            || tag.ends_with("-US")
    }) {
        &BUILTIN_FORMATS_US
    } else {
        &BUILTIN_FORMATS_CN
    }
}

/// 返回与 Java `switchBuiltinFormatsMap` 等价的格式到索引映射。
#[must_use]
pub fn switch_builtin_formats_map(
    locale: Option<&ExcelLocale>,
) -> &'static HashMap<&'static str, u16> {
    if std::ptr::eq(switch_builtin_formats_for_locale(locale), &BUILTIN_FORMATS_US) {
        &BUILTIN_FORMATS_MAP_US
    } else {
        &BUILTIN_FORMATS_MAP_CN
    }
}

/// The "all languages" built-in format table. (Java
/// `BUILTIN_FORMATS_ALL_LANGUAGES`)
/// 对应 Java：com.alibaba.excel.constant.BuiltinFormats。
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
/// 对应 Java：com.alibaba.excel.constant.BuiltinFormats。
pub static BUILTIN_FORMATS_CN: [Option<&str>; 82] = [
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
    Some("t0"),
    Some("t0.00"),
    Some("t#,##0"),
    Some("t#,##0.00"),
    None,
    None,
    None,
    None,
    Some("t0%"),
    Some("t0.00%"),
    Some("t# ?/?"),
    Some("t# ??/??"),
    Some("ว/ด/ปปปป"),
    Some("ว-ดดด-ปป"),
    Some("ว-ดดด"),
    Some("ดดด-ปป"),
    Some("ช:นน"),
    Some("ช:นน:ทท"),
    Some("ว/ด/ปปปป ช:นน"),
    Some("นน:ทท"),
    Some("[ช]:นน:ทท"),
    Some("นน:ทท.0"),
    Some("d/m/bb"),
];

/// 美国区域内建格式表。除货币符号外与 Java 的 US 表逐索引一致。
pub static BUILTIN_FORMATS_US: [Option<&str>; 82] = [
    Some("General"), Some("0"), Some("0.00"), Some("#,##0"), Some("#,##0.00"),
    Some("\"$\"#,##0_);(\"$\"#,##0)"), Some("\"$\"#,##0_);[Red](\"$\"#,##0)"),
    Some("\"$\"#,##0.00_);(\"$\"#,##0.00)"), Some("\"$\"#,##0.00_);[Red](\"$\"#,##0.00)"),
    Some("0%"), Some("0.00%"), Some("0.00E+00"), Some("# ?/?"), Some("# ??/??"),
    Some("yyyy/m/d"), Some("d-mmm-yy"), Some("d-mmm"), Some("mmm-yy"), Some("h:mm AM/PM"),
    Some("h:mm:ss AM/PM"), Some("h:mm"), Some("h:mm:ss"), Some("yyyy-m-d h:mm"),
    None, None, None, None,
    Some("yyyy\"年\"m\"月\""), Some("m\"月\"d\"日\""), Some("m\"月\"d\"日\""), Some("m-d-yy"),
    Some("yyyy\"年\"m\"月\"d\"日\""), Some("h\"时\"mm\"分\""), Some("h\"时\"mm\"分\"ss\"秒\""),
    Some("上午/下午h\"时\"mm\"分\""), Some("上午/下午h\"时\"mm\"分\"ss\"秒\""), Some("yyyy\"年\"m\"月\""),
    Some("#,##0_);(#,##0)"), Some("#,##0_);[Red](#,##0)"), Some("#,##0.00_);(#,##0.00)"),
    Some("#,##0.00_);[Red](#,##0.00)"), Some("_(* #,##0_);_(* (#,##0);_(* \"-\"_);_(@_)"),
    Some("_(\"$\"* #,##0_);_(\"$\"* (#,##0);_(\"$\"* \"-\"_);_(@_)"),
    Some("_(* #,##0.00_);_(* (#,##0.00);_(* \"-\"??_);_(@_)"),
    Some("_(\"$\"* #,##0.00_);_(\"$\"* (#,##0.00);_(\"$\"* \"-\"??_);_(@_)"),
    Some("mm:ss"), Some("[h]:mm:ss"), Some("mm:ss.0"), Some("##0.0E+0"), Some("@"),
    Some("yyyy\"年\"m\"月\""), Some("m\"月\"d\"日\""), Some("yyyy\"年\"m\"月\""),
    Some("m\"月\"d\"日\""), Some("m\"月\"d\"日\""), Some("上午/下午h\"时\"mm\"分\""),
    Some("上午/下午h\"时\"mm\"分\"ss\"秒\""), Some("yyyy\"年\"m\"月\""), Some("m\"月\"d\"日\""),
    Some("t0"), Some("t0.00"), Some("t#,##0"), Some("t#,##0.00"), None, None, None, None,
    Some("t0%"), Some("t0.00%"), Some("t# ?/?"), Some("t# ??/??"), Some("ว/ด/ปปปป"),
    Some("ว-ดดด-ปป"), Some("ว-ดดด"), Some("ดดด-ปป"), Some("ช:นน"), Some("ช:นน:ทท"),
    Some("ว/ด/ปปปป ช:นน"), Some("นน:ทท"), Some("[ช]:นน:ทท"), Some("นน:ทท.0"), Some("d/m/bb"),
];

fn build_map(formats: &'static [Option<&'static str>]) -> HashMap<&'static str, u16> {
    formats
        .iter()
        .enumerate()
        .filter_map(|(index, format)| format.map(|value| (value, index as u16)))
        .collect()
}

/// 中国区域格式到索引的不可变映射。
pub static BUILTIN_FORMATS_MAP_CN: LazyLock<HashMap<&'static str, u16>> =
    LazyLock::new(|| build_map(&BUILTIN_FORMATS_CN));

/// 美国区域格式到索引的不可变映射。
pub static BUILTIN_FORMATS_MAP_US: LazyLock<HashMap<&'static str, u16>> =
    LazyLock::new(|| build_map(&BUILTIN_FORMATS_US));

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
