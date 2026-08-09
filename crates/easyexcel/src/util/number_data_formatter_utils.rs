//! Java `NumberDataFormatterUtils` 兼容入口。

use bigdecimal::BigDecimal;

use crate::metadata::GlobalConfiguration;
use crate::read::ExcelLocale;

/// 按全局配置格式化数字或 Excel 日期序列。
///
/// 对应 Java：`NumberDataFormatterUtils#format(BigDecimal, Short, String,
/// GlobalConfiguration)`。
#[must_use]
pub fn format(
    data: &BigDecimal,
    data_format: Option<i16>,
    data_format_string: Option<&str>,
    global_configuration: Option<&GlobalConfiguration>,
) -> String {
    let (use_1904_windowing, locale, use_scientific_format) = global_configuration.map_or(
        (None, None, None),
        |configuration| {
            (
                Some(configuration.use_1904windowing()),
                resolve_locale(configuration.locale()),
                Some(configuration.use_scientific_format()),
            )
        },
    );
    format_with_options(
        data,
        data_format,
        data_format_string,
        use_1904_windowing,
        locale,
        use_scientific_format,
    )
}

/// 按显式日期窗口、区域和科学计数策略格式化。
///
/// 对应 Java：`NumberDataFormatterUtils#format(BigDecimal, Short, String,
/// Boolean, Locale, Boolean)`。
#[must_use]
pub fn format_with_options(
    data: &BigDecimal,
    data_format: Option<i16>,
    data_format_string: Option<&str>,
    use_1904_windowing: Option<bool>,
    locale: Option<ExcelLocale>,
    use_scientific_format: Option<bool>,
) -> String {
    easyexcel_format::format_number_data(
        data,
        data_format,
        data_format_string,
        use_1904_windowing,
        locale,
        use_scientific_format,
    )
}

/// 清除当前线程缓存的 `DataFormatter`。
///
/// 对应 Java：`NumberDataFormatterUtils#removeThreadLocalCache()`。
pub fn remove_thread_local_cache() {
    easyexcel_format::remove_thread_local_cache();
}

fn resolve_locale(locale: &str) -> Option<ExcelLocale> {
    (!locale.eq_ignore_ascii_case("default"))
        .then(|| ExcelLocale::from_name(locale))
        .flatten()
}
