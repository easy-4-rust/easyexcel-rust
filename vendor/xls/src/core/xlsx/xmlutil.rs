//! Small helpers shared by the XLSX reader/writer for quick-xml.

use quick_xml::events::{BytesEnd, BytesStart};
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
                a.unescape_value()
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| String::from_utf8_lossy(&a.value).into_owned()),
            );
        }
    }
    None
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
