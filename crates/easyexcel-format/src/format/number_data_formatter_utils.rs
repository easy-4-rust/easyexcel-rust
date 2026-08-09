//! Java `NumberDataFormatterUtils` 的格式引擎实现。

use std::cell::RefCell;

use bigdecimal::BigDecimal;

use super::{DataFormatter, ExcelLocale};

thread_local! {
    /// 与 Java `DATA_FORMATTER_THREAD_LOCAL` 一致：同一线程首次调用时固定
    /// DataFormatter 配置，直到显式清理。
    static DATA_FORMATTER: RefCell<Option<DataFormatter>> = const { RefCell::new(None) };
}

/// 使用线程级 `DataFormatter` 格式化数字或 Excel 日期序列。
///
/// 对应 Java：`NumberDataFormatterUtils#format(BigDecimal, Short, String,
/// Boolean, Locale, Boolean)`。同一线程后续调用复用第一次的 locale 与日期
/// 窗口配置，调用 [`remove_thread_local_cache`] 后重新初始化。
#[must_use]
pub fn format_number_data(
    data: &BigDecimal,
    data_format: Option<i16>,
    data_format_string: Option<&str>,
    use_1904_windowing: Option<bool>,
    locale: Option<ExcelLocale>,
    use_scientific_format: Option<bool>,
) -> String {
    DATA_FORMATTER.with(|slot| {
        let mut slot = slot.borrow_mut();
        let formatter = slot.get_or_insert_with(|| {
            DataFormatter::new(use_1904_windowing, locale, use_scientific_format)
        });
        formatter.format(data, data_format, data_format_string)
    })
}

/// 清除当前线程缓存的数字格式器。
///
/// 对应 Java：`NumberDataFormatterUtils#removeThreadLocalCache()`。
pub fn remove_thread_local_cache() {
    DATA_FORMATTER.with(|slot| {
        slot.borrow_mut().take();
    });
}
