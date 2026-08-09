//! 对应 Java：`com.alibaba.excel.metadata.format.DataFormatter`.
//!
//! Java's 874-line class formats Excel numbers and dates using POI's
//! internal format engine. Rust delegates to the `ssfmt` crate at the
//! reader call site (`easyexcel-reader`), then applies
//! [`java_compat_format_code`] + [`java_compat_display`] so STRING mode
//! matches `EasyExcel` / POI.

use std::collections::HashMap;
use std::sync::Arc;

use bigdecimal::BigDecimal;
use ssfmt::{DateSystem, FormatOptions, Locale, NumberFormat};

use super::{ExcelLocale, NumberRoundingMode};

/// `ssfmt` 使用的区域设置类型。
pub use ssfmt::Locale as SpreadsheetLocale;

/// 已编译的 Excel 数字格式。
///
/// 解析格式代码会构建 AST；读取器应在工作表生命周期内复用该对象，避免逐单元格
/// 重复解析和克隆。对应 Java：`DataFormatter` 内部格式缓存。
#[derive(Debug, Clone, PartialEq)]
pub struct CompiledExcelFormat {
    format: NumberFormat,
    is_date: bool,
}

type CustomNumberFormat = Arc<dyn Fn(&BigDecimal) -> String + Send + Sync>;

/// 可复用的工作簿数字格式器。
///
/// 对应 Java：`com.alibaba.excel.metadata.format.DataFormatter`。格式 AST、
/// locale、日期窗口、默认回退格式与 `addFormat` 覆盖项全部由格式引擎拥有，
/// 上层门面只负责重导出与 Java 参数命名适配。
pub struct DataFormatter {
    use_1904_windowing: bool,
    locale: ExcelLocale,
    use_scientific_format: bool,
    default_number_format: Option<CustomNumberFormat>,
    custom_formats: HashMap<String, CustomNumberFormat>,
}

impl DataFormatter {
    /// 解析 Excel 舍入模式；未指定时使用 Java 默认的 HALF_UP。
    #[must_use]
    pub fn set_excel_style_rounding_mode(
        rounding_mode: Option<NumberRoundingMode>,
    ) -> NumberRoundingMode {
        rounding_mode.unwrap_or(NumberRoundingMode::HalfUp)
    }

    /// 使用 Java nullable 构造参数创建格式器。
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

    /// 按格式索引和格式代码渲染任意精度数字。
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
                    .and_then(resolve_builtin_format_code)
            })
            .unwrap_or("General");

        if let Some(formatter) = self.custom_formats.get(format_code) {
            return formatter(data);
        }

        let decimal_text = data.to_string();
        let value = decimal_text.parse::<f64>().unwrap_or_else(|_| {
            if decimal_text.starts_with('-') {
                f64::NEG_INFINITY
            } else {
                f64::INFINITY
            }
        });
        if format_code.eq_ignore_ascii_case("General") || format_code == "@" {
            return super::format_general_with_options(
                value,
                self.use_scientific_format,
                self.locale.formatter().decimal_separator,
            );
        }
        format_with_code(
            value,
            format_code,
            self.use_1904_windowing,
            &self.locale.formatter(),
        )
        .or_else(|| self.default_number_format.as_ref().map(|formatter| formatter(data)))
        .unwrap_or(decimal_text)
    }

    /// 设置无法解析格式代码时的回退格式器。
    pub fn set_default_number_format<F>(&mut self, formatter: F)
    where
        F: Fn(&BigDecimal) -> String + Send + Sync + 'static,
    {
        self.default_number_format = Some(Arc::new(formatter));
    }

    /// 注册或替换一个格式代码对应的自定义格式器。
    pub fn add_format<F>(&mut self, excel_format_string: impl Into<String>, formatter: F)
    where
        F: Fn(&BigDecimal) -> String + Send + Sync + 'static,
    {
        self.custom_formats
            .insert(excel_format_string.into(), Arc::new(formatter));
    }

    /// 返回是否使用 1904 日期窗口。
    #[must_use]
    pub const fn use_1904_windowing(&self) -> bool {
        self.use_1904_windowing
    }

    /// 返回区域设置。
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

impl CompiledExcelFormat {
    /// 返回该格式是否包含日期或时间字段。
    #[must_use]
    pub const fn is_date_format(&self) -> bool {
        self.is_date
    }
}

/// 编译 Excel 格式代码，供同一工作簿中的多个单元格复用。
#[must_use]
pub fn compile_format_code(code: &str) -> Option<CompiledExcelFormat> {
    let original = NumberFormat::parse(code).ok()?;
    let is_date = original.is_date_format();
    let resolved = if is_date {
        java_compat_date_format_code(code)
    } else {
        java_compat_format_code(code)
    };
    let format = NumberFormat::parse(&resolved).ok()?;
    Some(CompiledExcelFormat { format, is_date })
}

/// 使用预编译格式渲染数字，避免热路径重复解析格式 AST。
#[must_use]
pub fn format_with_compiled(
    value: f64,
    compiled: &CompiledExcelFormat,
    date_1904: bool,
    locale: &Locale,
) -> String {
    let options = FormatOptions {
        date_system: if date_1904 {
            DateSystem::Date1904
        } else {
            DateSystem::Date1900
        },
        locale: locale.clone(),
    };
    java_compat_display(&compiled.format.format(value, &options))
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 按 `EasyExcel` 优先级解析内建数字格式代码。
///
/// 先使用 `EasyExcel`/POI 兼容表，再回退到 ECMA-376 内建表。
#[must_use]
pub fn resolve_builtin_format_code(id: u32) -> Option<&'static str> {
    u16::try_from(id)
        .ok()
        .and_then(super::builtin_format_code)
        .or_else(|| ssfmt::format_code_from_id(id))
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 判断 Excel 数字格式代码是否表示日期或日期时间。
#[must_use]
pub fn is_date_format_code(code: &str) -> bool {
    NumberFormat::parse(code)
        .ok()
        .is_some_and(|format| format.is_date_format())
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 Strip orphan decimal points left by optional `#` fraction digits.
///
/// ssfmt/SSF keeps a trailing `.` for `#.##` / `#.##%` when the fractional
/// part is empty (`9999.%`). POI / `EasyExcel` STRING mode drops it (`9999%`).
///
/// Does **not** trim whitespace: Excel format codes may emit intentional
/// trailing spaces (e.g. negative section `\-0.00\ ` → `-1.07 `).
///
/// Currency glyphs (`￥` U+FFE5 vs `¥` U+00A5) are left as emitted by the
/// format code / BIFF string — callers must decode FORMAT records as Latin-1
/// compressed Unicode, not UTF-8, so `0xA5` stays `¥`.
#[must_use]
pub fn java_compat_display(value: &str) -> String {
    let mut out = value.replace(".%", "%");
    if out.ends_with('.') {
        out.pop();
    }
    out
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 Rewrite Excel date format codes so ssfmt matches POI / `EasyExcel` STRING.
///
/// - CN literal `上午/下午` → `AM/PM` token (locale supplies `上午`/`下午`).
/// - `mmmmm` (first-letter month) → POI private-use wrap around short month:
///   `"\u{E001}"mmm"\u{E002}"` (e.g. `\u{E001}1月\u{E002}`).
///
/// Does not alter quoted literals beyond the explicit mappings above, and does
/// **not** trim or rewrite currency symbols.
#[must_use]
pub fn java_compat_date_format_code(format_str: &str) -> String {
    // CN AM/PM is a literal slash-pair in BuiltinFormats / custom codes; ssfmt
    // only treats the ASCII `AM/PM` token as a day-period field.
    let with_ampm = format_str.replace("上午/下午", "AM/PM");
    // Replace longest `mmmmm` runs first so we never leave a bare `mmm` behind
    // from a partial match. POI wraps the short month with U+E001 / U+E002.
    with_ampm.replace("mmmmm", "\"\u{E001}\"mmm\"\u{E002}\"")
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 Clean a numeric format code the way `EasyExcel`
/// `DataFormatter.cleanFormatForNumber` does before `DecimalFormat`.
///
/// - Drop `_X` / `*X` alignment pads (ssfmt would otherwise emit a space;
///   POI / `EasyExcel` do not for STRING).
/// - Unescape `\` / `"` so literal spaces like `\ ` survive as trailing
///   spaces on negative accounting formats (`-1.07 `).
///
/// Date formats should **not** go through this helper (`EasyExcel` keeps
/// them on the `CellFormat` path). Callers must gate on date vs number.
#[must_use]
pub fn java_compat_format_code(format_str: &str) -> String {
    let mut sb: Vec<char> = format_str.chars().collect();

    // Pass 1: remove `_` / `*` spacers and the following pad character.
    let mut i = 0usize;
    while i < sb.len() {
        let c = sb[i];
        if (c == '_' || c == '*') && !(i > 0 && sb[i - 1] == '\\') {
            if i + 1 < sb.len() {
                sb.remove(i + 1);
            }
            sb.remove(i);
            continue;
        }
        i += 1;
    }

    // Pass 2: drop quotes / backslashes; strip `+` after `E` (engineering).
    let mut i = 0usize;
    while i < sb.len() {
        let c = sb[i];
        if c == '\\' || c == '"' {
            sb.remove(i);
            continue;
        }
        if c == '+' && i > 0 && sb[i - 1] == 'E' {
            sb.remove(i);
            continue;
        }
        i += 1;
    }

    sb.into_iter().collect()
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 使用 Excel 格式代码和 locale 渲染数字，并应用 POI/EasyExcel 兼容清理。
#[must_use]
pub fn format_with_code(
    value: f64,
    code: &str,
    date_1904: bool,
    locale: &Locale,
) -> Option<String> {
    let compiled = compile_format_code(code)?;
    Some(format_with_compiled(value, &compiled, date_1904, locale))
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 将浮点值约束为 Excel 显示使用的 14 位有效精度。
#[must_use]
pub fn excel_display_number(value: f64) -> f64 {
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    // 小于 10^14 的整数/半整数最多具有 15 位有效数字，而且二进制表示精确；
    // 经过 `%.14e` 再解析不会改变结果。跳过临时 String 和二次浮点解析，
    // 覆盖 Excel 中最常见的 ID、日期序号和 .5 步进数值。
    if value.abs() < 1E14 && (value * 2.0).fract() == 0.0 {
        return value;
    }
    format!("{value:.14e}").parse().unwrap_or(value)
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 判断 General 格式是否进入 Java `EasyExcel` 的极值科学计数分支。
#[must_use]
pub fn is_scientific_magnitude(value: f64) -> bool {
    let absolute = value.abs();
    absolute >= 1E11 || (absolute <= 1E-10 && absolute > 0.0)
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 Java General 在禁用科学计数时对极值的整数显示。
#[must_use]
pub fn java_plain_extreme_format(value: f64) -> String {
    let rounded = value.round();
    if rounded == 0.0 {
        "0".to_owned()
    } else {
        format!("{rounded:.0}")
    }
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 Java General 科学计数显示，保留 locale 小数分隔符。
#[must_use]
pub fn java_scientific_format(value: f64, decimal_separator: char) -> String {
    let formatted = format!("{value:.5e}");
    let Some((mantissa, exponent)) = formatted.split_once('e') else {
        return formatted;
    };
    let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
    let Ok(exponent) = exponent.parse::<i32>() else {
        return formatted;
    };
    let mantissa = if decimal_separator == '.' {
        mantissa.to_owned()
    } else {
        mantissa.replace('.', &decimal_separator.to_string())
    };
    format!("{mantissa}E{exponent}")
}

/// 对应 Java：com.alibaba.excel.metadata.format.DataFormatter。 Formats a numeric value using a built-in or custom Excel format
/// code. (Java `DataFormatter.formatRawCellContents(...)`)
///
#[must_use]
pub fn format_raw_cell_contents(value: f64, format_code: &str) -> Option<String> {
    format_with_code(value, format_code, false, &Locale::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_compat_display_strips_orphan_decimal_before_percent() {
        assert_eq!(java_compat_display("9999.%"), "9999%");
        assert_eq!(java_compat_display("9999."), "9999");
        // Intentional trailing space from format codes must be preserved.
        assert_eq!(java_compat_display("-1.07 "), "-1.07 ");
    }

    #[test]
    fn java_compat_format_code_strips_pads_keeps_literal_space() {
        // Positive `_ ` pad removed; negative `\ ` becomes trailing space.
        let cleaned = java_compat_format_code(r"0.00_ ;[Red]\-0.00\ ");
        assert_eq!(cleaned, "0.00;[Red]-0.00 ");
        // Accounting `_)` pad removed; `\(` / `\)` unescaped.
        let acct = java_compat_format_code(r"0.00_);[Red]\(0.00\)");
        assert_eq!(acct, "0.00;[Red](0.00)");
    }

    #[test]
    fn java_compat_date_format_code_rewrites_cn_ampm_and_mmmmm() {
        assert_eq!(
            java_compat_date_format_code(r#"[DBNum1]上午/下午h"时"mm"分""#),
            r#"[DBNum1]AM/PMh"时"mm"分""#
        );
        assert_eq!(
            java_compat_date_format_code("mmmmm/yy"),
            "\"\u{E001}\"mmm\"\u{E002}\"/yy"
        );
        // Trailing spaces in unrelated date codes must be preserved by callers;
        // this helper only rewrites the two known tokens.
        assert_eq!(
            java_compat_date_format_code(r#"yyyy"年"m"月" "#),
            r#"yyyy"年"m"月" "#
        );
    }

    #[test]
    fn exact_integer_and_half_fast_path_matches_java_precision_round_trip() {
        for value in [
            -99_999_999_999_999.5,
            -1_000_000.0,
            -0.5,
            0.5,
            42.0,
            999_999.5,
            99_999_999_999_999.5,
        ] {
            let java_round_trip = format!("{value:.14e}").parse::<f64>().unwrap();
            assert_eq!(excel_display_number(value), java_round_trip);
        }
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn format_code_strips_star_pad_and_engineering_plus() {
        // 对应 Java：DataFormatter 数字格式清理
        assert_eq!(java_compat_format_code(r"* #,##0"), "#,##0");
        assert_eq!(java_compat_format_code(r"0.00E+00"), "0.00E00");
        // 转义的下划线不作为填充符移除（仅去除反斜杠）
        assert_eq!(java_compat_format_code(r"\_0"), "_0");
    }

    #[test]
    fn format_raw_cell_contents_stub_returns_none() {
        // 对应 Java：格式化由 easyexcel-reader 的 ssfmt 完成
        assert_eq!(format_raw_cell_contents(1.5, "0.00"), None);
    }
}
