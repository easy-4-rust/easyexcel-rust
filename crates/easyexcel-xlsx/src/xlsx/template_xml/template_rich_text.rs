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
