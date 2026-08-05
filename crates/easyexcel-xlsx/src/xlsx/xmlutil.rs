//! Small helpers shared by the XLSX reader/writer for quick-xml.

use quick_xml::XmlVersion;
use quick_xml::escape::resolve_predefined_entity;
use quick_xml::events::{BytesEnd, BytesRef, BytesStart, BytesText};
use quick_xml::name::QName;

/// The local (namespace-stripped) name of a start element. Keeps original case
/// but strips any `prefix:` so Strict and Transitional namespaces both match.
pub fn local_name(e: &BytesStart) -> String {
    local_of(e.name())
}

/// Same as [`local_name`] but for end elements.
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

/// Fetch an attribute value by local name (ignoring any namespace prefix),
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
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&a.value).into_owned()),
            );
        }
    }
    None
}

/// 解码并展开文本节点中的 XML 预定义实体。
pub fn text(e: &BytesText<'_>) -> String {
    let decoded = e.xml_content(XmlVersion::Implicit1_0).unwrap_or_default();
    quick_xml::escape::unescape(&decoded)
        .map(|value| value.into_owned())
        .unwrap_or_else(|_| decoded.into_owned())
}

/// 解码 XML 字符引用或预定义实体引用。
///
/// quick-xml 0.41 会把 `&amp;`、`&quot;` 和 `&#...;` 作为独立的
/// `GeneralRef` 事件返回，因此读取公式、共享字符串等文本时必须把该事件
/// 与相邻的 `Text` 事件重新拼接。未知实体保持原样，避免静默丢失内容。
pub fn general_ref(e: &BytesRef<'_>) -> String {
    if let Ok(Some(character)) = e.resolve_char_ref() {
        return character.to_string();
    }

    let name = e.xml_content(XmlVersion::Implicit1_0).unwrap_or_default();
    resolve_predefined_entity(&name)
        .map(str::to_owned)
        .unwrap_or_else(|| format!("&{name};"))
}

/// XML-escape text content (`&`, `<`, `>`, `"`).
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

/// Does the string have leading or trailing whitespace requiring
/// `xml:space="preserve"`?
pub fn needs_preserve(s: &str) -> bool {
    s.starts_with([' ', '\t', '\n', '\r']) || s.ends_with([' ', '\t', '\n', '\r'])
}
