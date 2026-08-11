use crate::xlsx::generation::{Color, FontFormatSpec, FormatScript, FormatUnderline};

/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。模板单元格使用的内联富文本。
///
/// 字体区间仍由门面按 Java UTF-16 语义解析；本对象只保存显示文本和已经由 XLSX
/// 引擎编码的 SpreadsheetML `inlineStr` 内容，避免门面持有 OOXML 细节。
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateRichText {
    text: String,
    inline_string_xml: String,
}

impl TemplateRichText {
    /// 从有序字体片段构建模板富文本。
    ///
    /// # Errors
    ///
    /// 片段为空、字体颜色或主题参数越界时返回格式错误。
    pub fn from_runs(runs: &[(FontFormatSpec, String)]) -> Result<Self> {
        if runs.is_empty() {
            return Err(Error::Xlsx(
                "template rich text requires at least one run".to_owned(),
            ));
        }
        let mut text = String::new();
        let mut inline_string_xml = String::from("<is>");
        for (font, value) in runs {
            text.push_str(value);
            inline_string_xml.push_str("<r>");
            inline_string_xml.push_str(&font_properties_xml(font)?);
            if needs_preserve(value) {
                let _ = write!(
                    inline_string_xml,
                    "<t xml:space=\"preserve\">{}</t>",
                    escape_xml(value)
                );
            } else {
                let _ = write!(inline_string_xml, "<t>{}</t>", escape_xml(value));
            }
            inline_string_xml.push_str("</r>");
        }
        inline_string_xml.push_str("</is>");
        Ok(Self {
            text,
            inline_string_xml,
        })
    }

    /// 返回无字体区间时的单片段模板富文本。
    #[must_use]
    pub fn plain(text: impl Into<String>) -> Self {
        let text = text.into();
        let escaped = escape_xml(&text);
        let inline_string_xml = if needs_preserve(&text) {
            format!("<is><r><t xml:space=\"preserve\">{escaped}</t></r></is>")
        } else {
            format!("<is><r><t>{escaped}</t></r></is>")
        };
        Self {
            text,
            inline_string_xml,
        }
    }

    /// 返回占位符参与混合文本替换时使用的显示文本。
    #[must_use]
    pub fn as_text(&self) -> &str {
        &self.text
    }

    /// 返回可直接放入 `<c t="inlineStr">` 的 `<is>` 内容。
    #[must_use]
    pub fn inline_string_xml(&self) -> &str {
        &self.inline_string_xml
    }
}

fn font_properties_xml(font: &FontFormatSpec) -> Result<String> {
    let mut xml = String::from("<rPr>");
    if let Some(name) = &font.name {
        let _ = write!(xml, "<rFont val=\"{}\"/>", escape_xml(name));
    }
    if let Some(size) = font.size {
        if !size.is_finite() {
            return Err(Error::Xlsx(format!(
                "template rich-text font size must be finite: {size}"
            )));
        }
        let _ = write!(xml, "<sz val=\"{size}\"/>");
    }
    boolean_property(&mut xml, "b", font.bold);
    boolean_property(&mut xml, "i", font.italic);
    boolean_property(&mut xml, "strike", font.strikeout);
    if let Some(color) = font.color {
        xml.push_str(&color_xml(color)?);
    }
    if let Some(script) = font.script {
        match script {
            FormatScript::None => {}
            FormatScript::Superscript => xml.push_str("<vertAlign val=\"superscript\"/>"),
            FormatScript::Subscript => xml.push_str("<vertAlign val=\"subscript\"/>"),
        }
    }
    if let Some(underline) = font.underline {
        let value = match underline {
            FormatUnderline::None => None,
            FormatUnderline::Single => Some("single"),
            FormatUnderline::Double => Some("double"),
            FormatUnderline::SingleAccounting => Some("singleAccounting"),
            FormatUnderline::DoubleAccounting => Some("doubleAccounting"),
        };
        if let Some(value) = value {
            let _ = write!(xml, "<u val=\"{value}\"/>");
        }
    }
    if let Some(charset) = font.charset {
        let _ = write!(xml, "<charset val=\"{charset}\"/>");
    }
    xml.push_str("</rPr>");
    Ok(xml)
}

fn boolean_property(xml: &mut String, tag: &str, value: Option<bool>) {
    match value {
        Some(true) => {
            let _ = write!(xml, "<{tag}/>");
        }
        Some(false) => {
            let _ = write!(xml, "<{tag} val=\"0\"/>");
        }
        None => {}
    }
}

fn color_xml(color: Color) -> Result<String> {
    let xml = match color {
        Color::Default => String::new(),
        Color::Automatic => "<color auto=\"1\"/>".to_owned(),
        Color::Theme(theme, shade) => {
            if theme > 9 || shade > 5 {
                return Err(Error::Xlsx(format!(
                    "template rich-text theme color ({theme}, {shade}) is outside XLSX limits"
                )));
            }
            theme_tint(theme, shade).map_or_else(
                || format!("<color theme=\"{theme}\"/>"),
                |tint| format!("<color theme=\"{theme}\" tint=\"{tint}\"/>"),
            )
        }
        other => format!("<color rgb=\"FF{}\"/>", rgb_hex(other)),
    };
    Ok(xml)
}

fn rgb_hex(color: Color) -> String {
    let value = match color {
        Color::RGB(value) => value,
        Color::Black | Color::Default | Color::Automatic | Color::Theme(_, _) => 0x000000,
        Color::Blue => 0x0000FF,
        Color::Brown => 0x800000,
        Color::Cyan => 0x00FFFF,
        Color::Gray => 0x808080,
        Color::Green => 0x008000,
        Color::Lime => 0x00FF00,
        Color::Magenta => 0xFF00FF,
        Color::Navy => 0x000080,
        Color::Orange => 0xFF6600,
        Color::Pink => 0xFFC0CB,
        Color::Purple => 0x800080,
        Color::Red => 0xFF0000,
        Color::Silver => 0xC0C0C0,
        Color::White => 0xFFFFFF,
        Color::Yellow => 0xFFFF00,
    };
    format!("{value:06X}")
}

fn theme_tint(theme: u8, shade: u8) -> Option<&'static str> {
    match (theme, shade) {
        (_, 0) => None,
        (0, 1) => Some("-4.9989318521683403E-2"),
        (0, 2) => Some("-0.14999847407452621"),
        (0, 3) => Some("-0.249977111117893"),
        (0, 4) => Some("-0.34998626667073579"),
        (0, 5) => Some("-0.499984740745262"),
        (1, 1) => Some("0.499984740745262"),
        (1, 2) => Some("0.34998626667073579"),
        (1, 3) => Some("0.249977111117893"),
        (1, 4) => Some("0.14999847407452621"),
        (1, 5) => Some("4.9989318521683403E-2"),
        (2, 1) => Some("-9.9978637043366805E-2"),
        (2, 2) => Some("-0.249977111117893"),
        (2, 3) => Some("-0.499984740745262"),
        (2, 4) => Some("-0.749992370372631"),
        (2, 5) => Some("-0.89999084444715716"),
        (_, 1) => Some("0.79998168889431442"),
        (_, 2) => Some("0.59999389629810485"),
        (_, 3) => Some("0.39997558519241921"),
        (_, 4) => Some("-0.249977111117893"),
        (_, 5) => Some("-0.499984740745262"),
        _ => None,
    }
}

fn needs_preserve(value: &str) -> bool {
    value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
}

#[cfg(test)]
mod template_rich_text_tests {
    use super::*;

    fn plain_font() -> FontFormatSpec {
        FontFormatSpec::default()
    }

    fn bold_font() -> FontFormatSpec {
        FontFormatSpec {
            bold: Some(true),
            ..FontFormatSpec::default()
        }
    }

    fn full_font() -> FontFormatSpec {
        FontFormatSpec {
            name: Some("Arial".to_owned()),
            size: Some(12.0),
            bold: Some(true),
            italic: Some(true),
            strikeout: Some(true),
            color: Some(Color::Red),
            script: Some(FormatScript::Superscript),
            underline: Some(FormatUnderline::Single),
            charset: Some(134),
        }
    }

    // --- from_runs 测试 ---

    /// 空 run 列表应返回错误。
    #[test]
    fn from_runs_empty_returns_error() {
        let result = TemplateRichText::from_runs(&[]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("at least one run")
        );
    }

    /// 单个普通 run 应生成正确 XML。
    #[test]
    fn from_runs_single_plain_run() {
        let rt = TemplateRichText::from_runs(&[(plain_font(), "hello".to_owned())]).unwrap();
        assert_eq!(rt.as_text(), "hello");
        assert_eq!(
            rt.inline_string_xml(),
            "<is><r><rPr></rPr><t>hello</t></r></is>"
        );
    }

    /// 多个 run 的文本拼接。
    #[test]
    fn from_runs_multiple_runs_concatenate_text() {
        let runs = vec![
            (plain_font(), "Hello".to_owned()),
            (bold_font(), " World".to_owned()),
        ];
        let rt = TemplateRichText::from_runs(&runs).unwrap();
        assert_eq!(rt.as_text(), "Hello World");
        assert!(rt.inline_string_xml().contains("<b/>"));
    }

    /// 前导/尾随空格触发 `xml:space="preserve"`。
    #[test]
    fn from_runs_leading_trailing_whitespace_preserves() {
        let rt = TemplateRichText::from_runs(&[(plain_font(), " space ".to_owned())]).unwrap();
        assert!(rt.inline_string_xml().contains("xml:space=\"preserve\""));
    }

    /// 无前后空格的文本不使用 preserve。
    #[test]
    fn from_runs_no_whitespace_no_preserve() {
        let rt = TemplateRichText::from_runs(&[(plain_font(), "hello".to_owned())]).unwrap();
        assert!(!rt.inline_string_xml().contains("xml:space=\"preserve\""));
    }

    /// 完整字体属性应生成所有 XML 标签。
    #[test]
    fn from_runs_full_font_generates_all_properties() {
        let rt = TemplateRichText::from_runs(&[(full_font(), "test".to_owned())]).unwrap();
        let xml = rt.inline_string_xml();
        assert!(xml.contains(r#"<rFont val="Arial"/>"#));
        assert!(xml.contains("<sz val=\"12\"/>"));
        assert!(xml.contains("<b/>"));
        assert!(xml.contains("<i/>"));
        assert!(xml.contains("<strike/>"));
        assert!(xml.contains("FFFF0000")); // Red → rgb_hex="FF0000" → rgb="FFFF0000"
        assert!(xml.contains("<vertAlign val=\"superscript\"/>"));
        assert!(xml.contains("<u val=\"single\"/>"));
        assert!(xml.contains("<charset val=\"134\"/>"));
    }

    /// 非有限字号应返回错误。
    #[test]
    fn from_runs_nan_font_size_returns_error() {
        let font = FontFormatSpec {
            size: Some(f64::NAN),
            ..FontFormatSpec::default()
        };
        let result = TemplateRichText::from_runs(&[(font, "text".to_owned())]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("font size must be finite")
        );
    }

    /// XML 特殊字符应被转义。
    #[test]
    fn from_runs_escapes_xml_special_chars() {
        let rt =
            TemplateRichText::from_runs(&[(plain_font(), "<tag>&\"'</tag>".to_owned())]).unwrap();
        let xml = rt.inline_string_xml();
        assert!(xml.contains("&lt;tag&gt;"));
        assert!(xml.contains("&amp;"));
        assert!(xml.contains("&quot;"));
        assert!(xml.contains("&apos;"));
    }

    // --- plain 测试 ---

    /// 纯文本应生成简洁 XML（无 rPr）。
    #[test]
    fn plain_text_creates_simple_xml() {
        let rt = TemplateRichText::plain("hello");
        assert_eq!(rt.as_text(), "hello");
        assert_eq!(
            rt.inline_string_xml(),
            "<is><r><t>hello</t></r></is>"
        );
    }

    /// 纯文本带空格时使用 preserve。
    #[test]
    fn plain_text_with_whitespace_uses_preserve() {
        let rt = TemplateRichText::plain(" hello ");
        assert!(rt.inline_string_xml().contains("xml:space=\"preserve\""));
    }

    /// 纯文本仅前导空格时使用 preserve。
    #[test]
    fn plain_text_leading_space_uses_preserve() {
        let rt = TemplateRichText::plain(" hello");
        assert!(rt.inline_string_xml().contains("xml:space=\"preserve\""));
    }

    /// 纯文本仅尾随空格时使用 preserve。
    #[test]
    fn plain_text_trailing_space_uses_preserve() {
        let rt = TemplateRichText::plain("hello ");
        assert!(rt.inline_string_xml().contains("xml:space=\"preserve\""));
    }

    /// 纯文本 XML 转义。
    #[test]
    fn plain_text_escapes_xml() {
        let rt = TemplateRichText::plain("a<b>c");
        assert!(rt.inline_string_xml().contains("a&lt;b&gt;c"));
    }

    // --- color_xml / rgb_hex 测试 ---

    /// Default 颜色产生空字符串。
    #[test]
    fn color_xml_default_is_empty() {
        let xml = color_xml(Color::Default).unwrap();
        assert!(xml.is_empty());
    }

    /// Automatic 颜色使用 auto 属性。
    #[test]
    fn color_xml_automatic() {
        let xml = color_xml(Color::Automatic).unwrap();
        assert_eq!(xml, "<color auto=\"1\"/>");
    }

    /// Theme 颜色 (0, 0) 无 tint。
    #[test]
    fn color_xml_theme_zero_shade() {
        let xml = color_xml(Color::Theme(0, 0)).unwrap();
        assert_eq!(xml, "<color theme=\"0\"/>");
    }

    /// Theme 颜色 (0, 1) 有 tint。
    #[test]
    fn color_xml_theme_with_tint() {
        let xml = color_xml(Color::Theme(0, 1)).unwrap();
        assert!(xml.contains("theme=\"0\""));
        assert!(xml.contains("tint="));
    }

    /// Theme 越界（theme > 9）返回错误。
    #[test]
    fn color_xml_invalid_theme_returns_error() {
        let result = color_xml(Color::Theme(10, 0));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("outside XLSX limits")
        );
    }

    /// Theme 越界（shade > 5）返回错误。
    #[test]
    fn color_xml_invalid_shade_returns_error() {
        let result = color_xml(Color::Theme(0, 6));
        assert!(result.is_err());
    }

    /// 预定义颜色生成正确的 RGB hex。
    #[test]
    fn rgb_hex_predefined_colors() {
        assert_eq!(rgb_hex(Color::Red), "FF0000");
        assert_eq!(rgb_hex(Color::Green), "008000");
        assert_eq!(rgb_hex(Color::Blue), "0000FF");
        assert_eq!(rgb_hex(Color::White), "FFFFFF");
        assert_eq!(rgb_hex(Color::Black), "000000");
        assert_eq!(rgb_hex(Color::Yellow), "FFFF00");
        assert_eq!(rgb_hex(Color::Cyan), "00FFFF");
        assert_eq!(rgb_hex(Color::Magenta), "FF00FF");
        assert_eq!(rgb_hex(Color::Navy), "000080");
        assert_eq!(rgb_hex(Color::Gray), "808080");
        assert_eq!(rgb_hex(Color::Silver), "C0C0C0");
        assert_eq!(rgb_hex(Color::Orange), "FF6600");
        assert_eq!(rgb_hex(Color::Pink), "FFC0CB");
        assert_eq!(rgb_hex(Color::Purple), "800080");
        assert_eq!(rgb_hex(Color::Lime), "00FF00");
        assert_eq!(rgb_hex(Color::Brown), "800000");
    }

    /// RGB 自定义颜色。
    #[test]
    fn rgb_hex_custom_rgb() {
        assert_eq!(rgb_hex(Color::RGB(0x123456)), "123456");
    }

    /// Default/Black/Automatic 等回退到 0x000000。
    #[test]
    fn rgb_hex_default_fallbacks() {
        assert_eq!(rgb_hex(Color::Default), "000000");
        assert_eq!(rgb_hex(Color::Automatic), "000000");
    }

    /// 预定义颜色在 color_xml 中生成 <color rgb="FF..." />。
    #[test]
    fn color_xml_rgb_predefined() {
        let xml = color_xml(Color::Red).unwrap();
        assert_eq!(xml, "<color rgb=\"FFFF0000\"/>");
    }

    // --- theme_tint 测试 ---

    /// shade=0 始终返回 None。
    #[test]
    fn theme_tint_shade_zero_returns_none() {
        assert_eq!(theme_tint(0, 0), None);
        assert_eq!(theme_tint(5, 0), None);
        assert_eq!(theme_tint(9, 0), None);
    }

    /// theme=0, shade=1 返回负 tint。
    #[test]
    fn theme_tint_dark1_shade1() {
        let tint = theme_tint(0, 1).unwrap();
        assert!(tint.starts_with('-'));
    }

    /// theme=1, shade=1 返回正 tint。
    #[test]
    fn theme_tint_light1_shade1() {
        let tint = theme_tint(1, 1).unwrap();
        assert!(tint.starts_with("0."));
    }

    /// 未知 theme (>2), shade 在 1-5 范围返回 fallback tint。
    #[test]
    fn theme_tint_unknown_theme_uses_fallback() {
        assert!(theme_tint(5, 1).is_some());
        assert!(theme_tint(5, 5).is_some());
    }

    /// shade > 5 且无匹配返回 None。
    #[test]
    fn theme_tint_out_of_range_returns_none() {
        assert_eq!(theme_tint(0, 6), None);
    }

    // --- needs_preserve 测试 ---

    /// 空字符串不需 preserve。
    #[test]
    fn needs_preserve_empty_string() {
        assert!(!needs_preserve(""));
    }

    /// 中间有空格但首尾无不需 preserve。
    #[test]
    fn needs_preserve_middle_whitespace_only() {
        assert!(!needs_preserve("a b"));
    }

    /// 前导空格需 preserve。
    #[test]
    fn needs_preserve_leading_space() {
        assert!(needs_preserve(" abc"));
    }

    /// 尾随空格需 preserve。
    #[test]
    fn needs_preserve_trailing_space() {
        assert!(needs_preserve("abc "));
    }

    /// 制表符也算空白。
    #[test]
    fn needs_preserve_tab() {
        assert!(needs_preserve("\tabc"));
        assert!(needs_preserve("abc\t"));
    }

    // --- boolean_property 测试 ---

    /// bold=true 写入 `<b/>`。
    #[test]
    fn boolean_property_true() {
        let mut xml = String::new();
        boolean_property(&mut xml, "b", Some(true));
        assert_eq!(xml, "<b/>");
    }

    /// bold=false 写入 `<b val="0"/>`。
    #[test]
    fn boolean_property_false() {
        let mut xml = String::new();
        boolean_property(&mut xml, "b", Some(false));
        assert_eq!(xml, r#"<b val="0"/>"#);
    }

    /// None 不写入。
    #[test]
    fn boolean_property_none() {
        let mut xml = String::new();
        boolean_property(&mut xml, "b", None);
        assert!(xml.is_empty());
    }

    // --- subscript 格式测试 ---

    /// 下标格式生成正确的 XML。
    #[test]
    fn from_runs_subscript_format() {
        let font = FontFormatSpec {
            script: Some(FormatScript::Subscript),
            ..FontFormatSpec::default()
        };
        let rt = TemplateRichText::from_runs(&[(font, "sub".to_owned())]).unwrap();
        assert!(rt.inline_string_xml().contains("<vertAlign val=\"subscript\"/>"));
    }

    /// 无下划线时不生成 <u> 标签。
    #[test]
    fn from_runs_no_underline_none() {
        let font = FontFormatSpec {
            underline: Some(FormatUnderline::None),
            ..FontFormatSpec::default()
        };
        let rt = TemplateRichText::from_runs(&[(font, "text".to_owned())]).unwrap();
        assert!(!rt.inline_string_xml().contains("<u "));
    }

    /// 双下划线格式。
    #[test]
    fn from_runs_double_underline() {
        let font = FontFormatSpec {
            underline: Some(FormatUnderline::Double),
            ..FontFormatSpec::default()
        };
        let rt = TemplateRichText::from_runs(&[(font, "text".to_owned())]).unwrap();
        assert!(rt.inline_string_xml().contains(r#"<u val="double"/>"#));
    }

    /// SingleAccounting 下划线。
    #[test]
    fn from_runs_single_accounting_underline() {
        let font = FontFormatSpec {
            underline: Some(FormatUnderline::SingleAccounting),
            ..FontFormatSpec::default()
        };
        let rt = TemplateRichText::from_runs(&[(font, "text".to_owned())]).unwrap();
        assert!(rt.inline_string_xml().contains(r#"<u val="singleAccounting"/>"#));
    }

    /// DoubleAccounting 下划线。
    #[test]
    fn from_runs_double_accounting_underline() {
        let font = FontFormatSpec {
            underline: Some(FormatUnderline::DoubleAccounting),
            ..FontFormatSpec::default()
        };
        let rt = TemplateRichText::from_runs(&[(font, "text".to_owned())]).unwrap();
        assert!(rt.inline_string_xml().contains(r#"<u val="doubleAccounting"/>"#));
    }

    /// Script::None 不生成 <vertAlign> 标签。
    #[test]
    fn from_runs_script_none() {
        let font = FontFormatSpec {
            script: Some(FormatScript::None),
            ..FontFormatSpec::default()
        };
        let rt = TemplateRichText::from_runs(&[(font, "text".to_owned())]).unwrap();
        assert!(!rt.inline_string_xml().contains("vertAlign"));
    }

    /// 无名称字体不生成 <rFont>。
    #[test]
    fn from_runs_no_font_name() {
        let font = FontFormatSpec {
            name: None,
            ..FontFormatSpec::default()
        };
        let rt = TemplateRichText::from_runs(&[(font, "text".to_owned())]).unwrap();
        assert!(!rt.inline_string_xml().contains("rFont"));
    }

    /// Theme 颜色 (2, 2) 使用 fallback tint。
    #[test]
    fn color_xml_theme2_shade2() {
        let xml = color_xml(Color::Theme(2, 2)).unwrap();
        assert!(xml.contains("theme=\"2\""));
        assert!(xml.contains("tint="));
    }

    /// 无穷大字号返回错误。
    #[test]
    fn from_runs_infinite_font_size_returns_error() {
        let font = FontFormatSpec {
            size: Some(f64::INFINITY),
            ..FontFormatSpec::default()
        };
        let result = TemplateRichText::from_runs(&[(font, "text".to_owned())]);
        assert!(result.is_err());
    }

    /// XML 特殊字符中 & 符号正确转义。
    #[test]
    fn escape_xml_ampersand() {
        assert_eq!(escape_xml("a&b"), "a&amp;b");
        assert_eq!(escape_xml("a<b"), "a&lt;b");
        assert_eq!(escape_xml("a>b"), "a&gt;b");
        assert_eq!(escape_xml("\"q\""), "&quot;q&quot;");
        assert_eq!(escape_xml("'q'"), "&apos;q&apos;");
    }
}
