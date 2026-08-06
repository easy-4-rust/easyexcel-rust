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

    fn display(
        &self,
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
        self.code()
            .and_then(|code| format_with_code(value, code, date_1904, locale))
    }
}

