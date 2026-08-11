/// 对应 Java：无直接对应对象；Rust 架构扩展。 XLSX 数字格式描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XlsxNumberFormat {
    /// 内建格式编号。
    Builtin(u32),
    /// 自定义格式代码。
    Custom(String),
}

impl XlsxNumberFormat {
    fn code(&self) -> Option<&str> {
        match self {
            Self::Builtin(id) => resolve_builtin_format_code(*id),
            Self::Custom(code) => Some(code.as_str()),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 判断是否为 General 格式。
    #[must_use]
    pub fn is_general(&self) -> bool {
        match self {
            Self::Builtin(id) => *id == 0,
            Self::Custom(code) => code.trim().eq_ignore_ascii_case("general"),
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 判断是否为日期或日期时间格式。
    #[must_use]
    pub fn is_date_format(&self) -> bool {
        self.code().is_some_and(is_date_format_code)
    }

    fn compile(&self) -> Option<CompiledExcelFormat> {
        self.code().and_then(compile_format_code)
    }

    fn display_compiled(
        &self,
        compiled: Option<&CompiledExcelFormat>,
        value: f64,
        date_1904: bool,
        use_scientific_format: bool,
        locale: &SpreadsheetLocale,
    ) -> Option<String> {
        if self.is_general() && is_scientific_magnitude(value) {
            return Some(if use_scientific_format {
                java_scientific_format(value, locale.decimal_separator)
            } else {
                java_plain_extreme_format(value)
            });
        }
        compiled.map(|compiled| format_with_compiled(value, compiled, date_1904, locale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── is_general 覆盖 ────────────────────────────────────────────────────

    #[test]
    fn is_general_builtin_zero() {
        let fmt = XlsxNumberFormat::Builtin(0);
        assert!(fmt.is_general());
    }

    #[test]
    fn is_general_custom_general_case_insensitive() {
        let fmt = XlsxNumberFormat::Custom("General".into());
        assert!(fmt.is_general());
    }

    #[test]
    fn is_general_custom_general_lowercase() {
        let fmt = XlsxNumberFormat::Custom("general".into());
        assert!(fmt.is_general());
    }

    #[test]
    fn is_general_custom_general_with_spaces() {
        let fmt = XlsxNumberFormat::Custom(" General ".into());
        assert!(fmt.is_general());
    }

    #[test]
    fn is_general_builtin_nonzero() {
        let fmt = XlsxNumberFormat::Builtin(1);
        assert!(!fmt.is_general());
    }

    #[test]
    fn is_general_custom_non_general() {
        let fmt = XlsxNumberFormat::Custom("0.00".into());
        assert!(!fmt.is_general());
    }

    // ── is_date_format 覆盖 ────────────────────────────────────────────────

    #[test]
    fn is_date_format_builtin_date() {
        // 内建格式 14 是日期格式 mm-dd-yy
        let fmt = XlsxNumberFormat::Builtin(14);
        assert!(fmt.is_date_format());
    }

    #[test]
    fn is_date_format_custom_date() {
        let fmt = XlsxNumberFormat::Custom("yyyy-mm-dd".into());
        assert!(fmt.is_date_format());
    }

    #[test]
    fn is_date_format_custom_datetime() {
        let fmt = XlsxNumberFormat::Custom("yyyy-mm-dd hh:mm:ss".into());
        assert!(fmt.is_date_format());
    }

    #[test]
    fn is_date_format_not_date() {
        let fmt = XlsxNumberFormat::Custom("0.00".into());
        assert!(!fmt.is_date_format());
    }

    #[test]
    fn is_date_format_general_not_date() {
        let fmt = XlsxNumberFormat::Builtin(0);
        assert!(!fmt.is_date_format());
    }

    // ── XlsxCellValue 覆盖 ─────────────────────────────────────────────────

    #[test]
    fn cell_value_empty_variant() {
        let v = XlsxCellValue::Empty;
        assert_eq!(v, XlsxCellValue::Empty);
    }

    #[test]
    fn cell_value_string_variant() {
        let v = XlsxCellValue::String("hello".into());
        assert_eq!(v, XlsxCellValue::String("hello".into()));
    }

    #[test]
    fn cell_value_bool_variant() {
        let v = XlsxCellValue::Bool(true);
        assert_eq!(v, XlsxCellValue::Bool(true));
    }

    #[test]
    fn cell_value_error_variant() {
        let v = XlsxCellValue::Error("#REF!".into());
        assert_eq!(v, XlsxCellValue::Error("#REF!".into()));
    }

    #[test]
    fn cell_value_number_variant() {
        let v = XlsxCellValue::Number(3.14);
        assert_eq!(v, XlsxCellValue::Number(3.14));
    }

    // ── XlsxExtraKind 覆盖 ─────────────────────────────────────────────────

    #[test]
    fn extra_kind_variants() {
        assert_ne!(XlsxExtraKind::Merge, XlsxExtraKind::Hyperlink);
        assert_ne!(XlsxExtraKind::Merge, XlsxExtraKind::Comment);
        assert_ne!(XlsxExtraKind::Hyperlink, XlsxExtraKind::Comment);
    }

    // ── XlsxDisplayOptions 默认值 ───────────────────────────────────────────

    #[test]
    fn display_options_default_values() {
        let opts = XlsxDisplayOptions::default();
        assert!(!opts.date_1904);
        assert!(!opts.use_scientific_format);
        assert!(opts.retain_decimal_values);
        assert!(opts.retain_display_columns.is_none());
    }
}
