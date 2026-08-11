//! Parse `xl/sharedStrings.xml` into a `Vec<String>`.

use quick_xml::Reader;
use quick_xml::events::Event;

use easyexcel_io::Result;

use super::xmlutil::{general_ref, local_name, local_name_end, text};

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Parse shared strings. Each `<si>` becomes one entry; rich-text runs
/// (`<r><t>...`) are concatenated. `xml:space="preserve"` is respected because
/// we capture raw text events without trimming.
pub fn parse_shared_strings(xml: &[u8]) -> Result<Vec<String>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut strings = Vec::new();
    let mut buf = Vec::new();

    let mut in_si = false;
    let mut in_t = false;
    let mut current = String::new();

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Start(e) => {
                let name = local_name(&e);
                match name.as_str() {
                    "si" => {
                        in_si = true;
                        current.clear();
                    }
                    "t" if in_si => in_t = true,
                    _ => {}
                }
            }
            Event::End(e) => {
                let name = local_name_end(&e);
                match name.as_str() {
                    "si" => {
                        in_si = false;
                        strings.push(std::mem::take(&mut current));
                    }
                    "t" => in_t = false,
                    _ => {}
                }
            }
            Event::Text(t) if in_t => {
                current.push_str(&text(&t));
            }
            Event::GeneralRef(reference) if in_t => {
                current.push_str(&general_ref(&reference));
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(strings)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 简单字符串解析。
    #[test]
    fn parse_simple_shared_strings() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="2" uniqueCount="2">
                <si><t>hello</t></si>
                <si><t>world</t></si>
            </sst>"#;
        let strings = parse_shared_strings(xml).unwrap();
        assert_eq!(strings, vec!["hello", "world"]);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 富文本多 run 拼接。
    #[test]
    fn parse_rich_text_shared_strings() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
                <si>
                    <r><t>Hello </t></r>
                    <r><t>World</t></r>
                </si>
            </sst>"#;
        let strings = parse_shared_strings(xml).unwrap();
        assert_eq!(strings, vec!["Hello World"]);
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 空共享字符串。
    #[test]
    fn parse_empty_shared_strings() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="0" uniqueCount="0">
            </sst>"#;
        let strings = parse_shared_strings(xml).unwrap();
        assert!(strings.is_empty());
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 空 `<si>` 元素产生空字符串。
    #[test]
    fn parse_empty_si_element() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
                <si><t/></si>
            </sst>"#;
        let strings = parse_shared_strings(xml).unwrap();
        assert_eq!(strings.len(), 1);
        assert_eq!(strings[0], "");
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 含 XML 实体的共享字符串。
    #[test]
    fn parse_shared_strings_with_entities() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="1">
                <si><t>a &amp; b</t></si>
            </sst>"#;
        let strings = parse_shared_strings(xml).unwrap();
        assert_eq!(strings, vec!["a & b"]);
    }
}
