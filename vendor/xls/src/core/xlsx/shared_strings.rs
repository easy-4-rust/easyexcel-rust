//! Parse `xl/sharedStrings.xml` into a `Vec<String>`.

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::core::error::Result;

use super::xmlutil::{local_name, local_name_end};

/// Parse shared strings. Each `<si>` becomes one entry; rich-text runs
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
                current.push_str(&t.unescape().unwrap_or_default());
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(strings)
}
