//! Java `BuiltinFormats` 兼容路径。

use std::collections::HashMap;

use easyexcel_format::ExcelLocale;
pub use easyexcel_format::{
    BUILTIN_FORMATS_ALL_LANGUAGES, BUILTIN_FORMATS_CN, BUILTIN_FORMATS_MAP_CN,
    BUILTIN_FORMATS_MAP_US, BUILTIN_FORMATS_US, GENERAL, MIN_CUSTOM_DATA_FORMAT_INDEX,
    builtin_format_code, get_builtin_format, get_builtin_format_for_locale, switch_builtin_formats,
    switch_builtin_formats_for_locale, switch_builtin_formats_map,
};

/// Java `BuiltinFormats` 的静态门面。
#[derive(Debug, Clone, Copy, Default)]
pub struct BuiltinFormats;

impl BuiltinFormats {
    /// 通用格式索引。
    pub const GENERAL: u16 = GENERAL;
    /// 自定义格式的最小索引。
    pub const MIN_CUSTOM_DATA_FORMAT_INDEX: u16 = MIN_CUSTOM_DATA_FORMAT_INDEX;
    /// 跨语言内建格式。
    pub const BUILTIN_FORMATS_ALL_LANGUAGES: &'static [Option<&'static str>] =
        &BUILTIN_FORMATS_ALL_LANGUAGES;
    /// 中国区域内建格式。
    pub const BUILTIN_FORMATS_CN: &'static [Option<&'static str>] = &BUILTIN_FORMATS_CN;
    /// 美国区域内建格式。
    pub const BUILTIN_FORMATS_US: &'static [Option<&'static str>] = &BUILTIN_FORMATS_US;

    /// 获取内建格式，完整复现 Java 的默认值与区域回退顺序。
    #[must_use]
    pub fn get_builtin_format<'a>(
        index: Option<u16>,
        default_format: Option<&'a str>,
        locale: Option<&ExcelLocale>,
    ) -> Option<&'a str> {
        get_builtin_format_for_locale(index, default_format, locale)
    }

    /// 按区域选择格式表。
    #[must_use]
    pub fn switch_builtin_formats(
        locale: Option<&ExcelLocale>,
    ) -> &'static [Option<&'static str>] {
        switch_builtin_formats_for_locale(locale)
    }

    /// 按区域选择格式索引映射。
    #[must_use]
    pub fn switch_builtin_formats_map(
        locale: Option<&ExcelLocale>,
    ) -> &'static HashMap<&'static str, u16> {
        switch_builtin_formats_map(locale)
    }
}
