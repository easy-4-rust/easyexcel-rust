//! BIFF8 内建数字格式码表。

/// 根据格式代码返回 BIFF8 内建 `ifmt`。
#[must_use]
pub(crate) fn builtin_format_id(code: &str) -> Option<u16> {
    BUILTIN_FORMATS
        .iter()
        .find_map(|(id, value)| (*value == code).then_some(*id))
}

/// 根据 BIFF8 内建 `ifmt` 返回格式代码。
#[must_use]
pub(crate) fn builtin_format_code(id: u16) -> Option<&'static str> {
    BUILTIN_FORMATS
        .iter()
        .find_map(|(value, code)| (*value == id).then_some(*code))
}

const BUILTIN_FORMATS: &[(u16, &str)] = &[
    (0, "General"),
    (1, "0"),
    (2, "0.00"),
    (3, "#,##0"),
    (4, "#,##0.00"),
    (5, "$#,##0_);($#,##0)"),
    (6, "$#,##0_);[Red]($#,##0)"),
    (7, "$#,##0.00_);($#,##0.00)"),
    (8, "$#,##0.00_);[Red]($#,##0.00)"),
    (9, "0%"),
    (10, "0.00%"),
    (11, "0.00E+00"),
    (12, "# ?/?"),
    (13, "# ??/??"),
    (14, "m/d/yy"),
    (15, "d-mmm-yy"),
    (16, "d-mmm"),
    (17, "mmm-yy"),
    (18, "h:mm AM/PM"),
    (19, "h:mm:ss AM/PM"),
    (20, "h:mm"),
    (21, "h:mm:ss"),
    (22, "m/d/yy h:mm"),
    (37, "#,##0_);(#,##0)"),
    (38, "#,##0_);[Red](#,##0)"),
    (39, "#,##0.00_);(#,##0.00)"),
    (40, "#,##0.00_);[Red](#,##0.00)"),
    (41, "_(* #,##0_);_(* (#,##0);_(* \"-\"_);_(@_)"),
    (43, "_(* #,##0.00_);_(* (#,##0.00);_(* \"-\"??_);_(@_)"),
    (45, "mm:ss"),
    (46, "[h]:mm:ss"),
    (47, "mm:ss.0"),
    (48, "##0.0E+0"),
    (49, "@"),
];
