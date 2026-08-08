//! Java `DataFormatter` 兼容对象。

use std::collections::HashMap;
use std::sync::Arc;

use bigdecimal::BigDecimal;

use crate::read::ExcelLocale;

pub use easyexcel_format::{
    format_raw_cell_contents, java_compat_date_format_code, java_compat_display,
    java_compat_format_code,
};

type CustomNumberFormat = Arc<dyn Fn(&BigDecimal) -> String + Send + Sync>;

/// 非线程安全的工作簿数字格式器。
///
/// 对应 Java：`com.alibaba.excel.metadata.format.DataFormatter`。格式 AST 与
/// locale 解析由 `easyexcel-format` 缓存层承担；本对象保存 Java 构造参数、
/// 默认回退格式以及用户通过 `addFormat` 注册的覆盖项。
pub struct DataFormatter {
    use_1904_windowing: bool,
    locale: ExcelLocale,
    use_scientific_format: bool,
    default_number_format: Option<CustomNumberFormat>,
    custom_formats: HashMap<String, CustomNumberFormat>,
}

impl DataFormatter {
    /// 解析 Excel 的舍入模式；未指定时使用 Java 默认的 HALF_UP。
    /// 对应 Java：`setExcelStyleRoundingMode` 两个重载。
    #[must_use]
    pub fn set_excel_style_rounding_mode(
        rounding_mode: Option<crate::NumberRoundingMode>,
    ) -> crate::NumberRoundingMode {
        rounding_mode.unwrap_or(crate::NumberRoundingMode::HalfUp)
    }
    /// 对应 Java 构造器。`None` 保留 nullable Boolean/Locale 的默认语义。
    #[must_use]
    pub fn new(
        use_1904_windowing: Option<bool>,
        locale: Option<ExcelLocale>,
        use_scientific_format: Option<bool>,
    ) -> Self {
        Self {
            use_1904_windowing: use_1904_windowing.unwrap_or(false),
            locale: locale.unwrap_or_default(),
            use_scientific_format: use_scientific_format.unwrap_or(false),
            default_number_format: None,
            custom_formats: HashMap::new(),
        }
    }

    /// 按 Excel 格式索引和格式字符串渲染任意精度数字。
    ///
    /// 对应 Java `format(BigDecimal, Short, String)`。
    #[must_use]
    pub fn format(
        &self,
        data: &BigDecimal,
        data_format: Option<i16>,
        data_format_string: Option<&str>,
    ) -> String {
        let format_code = data_format_string
            .filter(|value| !value.is_empty())
            .or_else(|| {
                data_format
                    .and_then(|value| u32::try_from(value).ok())
                    .and_then(easyexcel_format::resolve_builtin_format_code)
            })
            .unwrap_or("General");

        if let Some(formatter) = self.custom_formats.get(format_code) {
            return formatter(data);
        }

        let value = data.to_string().parse::<f64>().unwrap_or_else(|_| {
            if data.to_string().starts_with('-') {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        });
        if format_code.eq_ignore_ascii_case("General") || format_code == "@" {
            return easyexcel_format::format_general_with_options(
                value,
                self.use_scientific_format,
                self.locale.formatter().decimal_separator,
            );
        }
        easyexcel_format::format_with_code(
            value,
            format_code,
            self.use_1904_windowing,
            &self.locale.formatter(),
        )
        .or_else(|| self.default_number_format.as_ref().map(|formatter| formatter(data)))
        .unwrap_or_else(|| data.to_string())
    }

    /// 设置无法解析格式代码时的回退格式器。对应 Java
    /// `setDefaultNumberFormat(Format)`。
    pub fn set_default_number_format<F>(&mut self, formatter: F)
    where
        F: Fn(&BigDecimal) -> String + Send + Sync + 'static,
    {
        self.default_number_format = Some(Arc::new(formatter));
    }

    /// 注册或替换 Excel 格式代码。对应 Java `addFormat(String, Format)`。
    pub fn add_format<F>(&mut self, excel_format_string: impl Into<String>, formatter: F)
    where
        F: Fn(&BigDecimal) -> String + Send + Sync + 'static,
    {
        self.custom_formats
            .insert(excel_format_string.into(), Arc::new(formatter));
    }

    /// 返回 1904 日期窗口配置。
    #[must_use]
    pub const fn use_1904_windowing(&self) -> bool {
        self.use_1904_windowing
    }

    /// 返回 locale。
    #[must_use]
    pub const fn locale(&self) -> &ExcelLocale {
        &self.locale
    }
}

impl Default for DataFormatter {
    fn default() -> Self {
        Self::new(None, None, None)
    }
}
