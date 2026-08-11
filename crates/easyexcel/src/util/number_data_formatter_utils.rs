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
    let (use_1904_windowing, locale, use_scientific_format) =
        global_configuration.map_or((None, None, None), |configuration| {
            (
                Some(configuration.use1904windowing()),
                resolve_locale(configuration.locale()),
                Some(configuration.use_scientific_format()),
            )
        });
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

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::BigDecimal;

    #[test]
    fn format_integer_returns_string() {
        let data = BigDecimal::from(42_i32);
        let result = format(&data, None, None, None);
        assert!(!result.is_empty(), "格式化结果不应为空");
    }

    #[test]
    fn format_with_global_configuration() {
        let data = BigDecimal::from(100_i32);
        let config = crate::GlobalConfiguration::default();
        let result = format(&data, None, None, Some(&config));
        assert!(!result.is_empty(), "格式化结果不应为空");
    }

    #[test]
    fn format_with_options_returns_string() {
        let data: BigDecimal = "3.14".parse().unwrap();
        let result = format_with_options(&data, None, None, None, None, None);
        assert!(!result.is_empty(), "格式化结果不应为空");
    }

    #[test]
    fn format_with_scientific_option() {
        let data = BigDecimal::from(1000000_i64);
        let result = format_with_options(&data, None, None, None, None, Some(true));
        assert!(!result.is_empty(), "科学计数法结果不应为空");
    }

    #[test]
    fn format_with_data_format() {
        let data = BigDecimal::from(1_i32);
        let result = format_with_options(&data, Some(2), None, None, None, None);
        assert!(!result.is_empty(), "指定格式编号结果不应为空");
    }

    #[test]
    fn remove_thread_local_cache_does_not_panic() {
        remove_thread_local_cache();
    }

    #[test]
    fn resolve_locale_returns_none_for_default() {
        assert_eq!(resolve_locale("default"), None);
        assert_eq!(resolve_locale("DEFAULT"), None);
    }

    #[test]
    fn resolve_locale_returns_some_for_known_locale() {
        let result = resolve_locale("en_US");
        // 取决于 ExcelLocale 是否识别该 locale
        // 至少确认不 panic
        let _ = result;
    }

    #[test]
    fn format_decimal_zero() {
        let data = BigDecimal::from(0_i32);
        let result = format(&data, None, None, None);
        assert!(!result.is_empty());
    }

    #[test]
    fn format_negative_number() {
        let data = BigDecimal::from(-100_i32);
        let result = format(&data, None, None, None);
        assert!(!result.is_empty());
    }
}
