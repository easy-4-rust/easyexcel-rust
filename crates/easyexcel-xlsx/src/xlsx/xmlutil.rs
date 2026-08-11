//! Small helpers shared by the XLSX reader/writer for quick-xml.

use std::collections::HashMap;

use quick_xml::XmlVersion;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesEnd, BytesRef, BytesStart, BytesText};
use quick_xml::name::QName;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回去除命名空间前缀后的 XML 标签名。
#[must_use]
pub fn local_tag_name(name: &str) -> &str {
    name.rsplit(':').next().unwrap_or(name)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 `EasyExcel` SAX 兼容层使用的空白分隔 `key=value` 属性袋。
///
/// 真正的 OOXML 事件读取由 `quick-xml` 完成；该格式用于把已解码属性传给
/// Java 风格 `XlsxTagHandler`，因此统一放在 XLSX 引擎层，避免各 handler
/// 重复实现字符串切分。
#[must_use]
pub fn parse_attribute_pairs(attributes: &str) -> HashMap<String, String> {
    attributes
        .split_whitespace()
        .filter_map(|token| token.split_once('='))
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 OOXML 文本中的 `_xHHHH_` 转义序列。
#[must_use]
pub fn decode_ooxml_escape(value: &str) -> String {
    if !value.contains("_x") {
        return value.to_owned();
    }
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut index = 0;
    while index < bytes.len() {
        if index + 7 <= bytes.len()
            && bytes[index] == b'_'
            && bytes[index + 1] == b'x'
            && bytes[index + 6] == b'_'
            && let Ok(hex) = std::str::from_utf8(&bytes[index + 2..index + 6])
            && let Ok(code) = u16::from_str_radix(hex, 16)
            && let Some(character) = char::from_u32(u32::from(code))
        {
            output.push(character);
            index += 7;
        } else if let Some(character) = value[index..].chars().next() {
            output.push(character);
            index += character.len_utf8();
        } else {
            break;
        }
    }
    output
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 The local (namespace-stripped) name of a start element. Keeps original case
/// but strips any `prefix:` so Strict and Transitional namespaces both match.
pub fn local_name(e: &BytesStart) -> String {
    local_of(e.name())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Same as [`local_name`] but for end elements.
pub fn local_name_end(e: &BytesEnd) -> String {
    local_of(e.name())
}

fn local_of(name: QName<'_>) -> String {
    let bytes = name.as_ref();
    let local = match bytes.iter().rposition(|&b| b == b':') {
        Some(pos) => &bytes[pos + 1..],
        None => bytes,
    };
    String::from_utf8_lossy(local).into_owned()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Fetch an attribute value by local name (ignoring any namespace prefix),
/// decoding entities.
pub fn attr(e: &BytesStart, name: &str) -> Option<String> {
    for a in e.attributes().with_checks(false).flatten() {
        let key = a.key.as_ref();
        let local = match key.iter().rposition(|&b| b == b':') {
            Some(pos) => &key[pos + 1..],
            None => key,
        };
        if local == name.as_bytes() {
            return Some(
                a.decoded_and_normalized_value(XmlVersion::Implicit1_0, e.decoder())
                    .map_or_else(
                        |_| String::from_utf8_lossy(&a.value).into_owned(),
                        std::borrow::Cow::into_owned,
                    ),
            );
        }
    }
    None
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码并展开文本节点中的 XML 预定义实体。
pub fn text(e: &BytesText<'_>) -> String {
    let decoded = e.xml_content(XmlVersion::Implicit1_0).unwrap_or_default();
    match quick_xml::escape::unescape(&decoded) {
        Ok(value) => value.into_owned(),
        Err(_) => decoded.into_owned(),
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 XML 字符引用或预定义实体引用。
///
/// quick-xml 0.41 会把 `&amp;`、`&quot;` 和 `&#...;` 作为独立的
/// `GeneralRef` 事件返回，因此读取公式、共享字符串等文本时必须把该事件
/// 与相邻的 `Text` 事件重新拼接。未知实体保持原样，避免静默丢失内容。
pub fn general_ref(e: &BytesRef<'_>) -> String {
    if let Ok(Some(character)) = e.resolve_char_ref() {
        return character.to_string();
    }

    let name = e.xml_content(XmlVersion::Implicit1_0).unwrap_or_default();
    resolve_predefined_entity(&name).map_or_else(|| format!("&{name};"), str::to_owned)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 XML-escape text content (`&`, `<`, `>`, `"`).
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Does the string have leading or trailing whitespace requiring
/// `xml:space="preserve"`?
pub fn needs_preserve(s: &str) -> bool {
    s.starts_with([' ', '\t', '\n', '\r']) || s.ends_with([' ', '\t', '\n', '\r'])
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── local_tag_name 覆盖 ────────────────────────────────────────────────

    #[test]
    fn local_tag_name_strips_namespace() {
        assert_eq!(local_tag_name("x:sheetData"), "sheetData");
        assert_eq!(local_tag_name("ns:workbook"), "workbook");
    }

    #[test]
    fn local_tag_name_returns_plain_name() {
        assert_eq!(local_tag_name("sheetData"), "sheetData");
    }

    // ── parse_attribute_pairs 覆盖 ─────────────────────────────────────────

    #[test]
    fn parse_attribute_pairs_parses_key_value() {
        let result = parse_attribute_pairs("name=Sheet1 rId=rId1");
        assert_eq!(result.get("name").unwrap(), "Sheet1");
        assert_eq!(result.get("rId").unwrap(), "rId1");
    }

    #[test]
    fn parse_attribute_pairs_handles_empty_string() {
        let result = parse_attribute_pairs("");
        assert!(result.is_empty());
    }

    #[test]
    fn parse_attribute_pairs_ignores_tokens_without_equals() {
        let result = parse_attribute_pairs("valid=x ignored");
        assert_eq!(result.len(), 1);
        assert_eq!(result.get("valid").unwrap(), "x");
    }

    // ── decode_ooxml_escape 覆盖 ───────────────────────────────────────────

    #[test]
    fn decode_ooxml_escape_converts_hex_sequence() {
        // _x0041_ = 'A'
        assert_eq!(decode_ooxml_escape("_x0041_"), "A");
    }

    #[test]
    fn decode_ooxml_escape_handles_no_escape() {
        assert_eq!(decode_ooxml_escape("plain text"), "plain text");
    }

    #[test]
    fn decode_ooxml_escape_handles_multiple_escapes() {
        // _x0041_ = 'A', _x0042_ = 'B'
        assert_eq!(decode_ooxml_escape("_x0041__x0042_"), "AB");
    }

    #[test]
    fn decode_ooxml_escape_handles_mixed_content() {
        assert_eq!(decode_ooxml_escape("hello_x0020_world"), "hello world");
    }

    #[test]
    fn decode_ooxml_escape_handles_invalid_hex_gracefully() {
        // _xGGGG_ 不是有效十六进制，保持原样
        assert_eq!(decode_ooxml_escape("_xGGGG_"), "_xGGGG_");
    }

    #[test]
    fn decode_ooxml_escape_handles_partial_escape() {
        // _x004 不完整序列
        assert_eq!(decode_ooxml_escape("_x004"), "_x004");
    }

    // ── xml_escape 覆盖 ────────────────────────────────────────────────────

    #[test]
    fn xml_escape_escapes_all_special_chars() {
        assert_eq!(
            xml_escape("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn xml_escape_handles_empty_string() {
        assert_eq!(xml_escape(""), "");
    }

    #[test]
    fn xml_escape_handles_no_special_chars() {
        assert_eq!(xml_escape("hello world"), "hello world");
    }

    // ── needs_preserve 覆盖 ────────────────────────────────────────────────

    #[test]
    fn needs_preserve_detects_leading_space() {
        assert!(needs_preserve(" hello"));
    }

    #[test]
    fn needs_preserve_detects_trailing_space() {
        assert!(needs_preserve("hello "));
    }

    #[test]
    fn needs_preserve_detects_leading_tab() {
        assert!(needs_preserve("\thello"));
    }

    #[test]
    fn needs_preserve_detects_trailing_newline() {
        assert!(needs_preserve("hello\n"));
    }

    #[test]
    fn needs_preserve_returns_false_for_clean_string() {
        assert!(!needs_preserve("hello"));
    }

    #[test]
    fn needs_preserve_returns_false_for_empty_string() {
        assert!(!needs_preserve(""));
    }
}
