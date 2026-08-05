//! OOXML ZIP 包路径与关系解析原语。
//!
//! 这些操作不包含 EasyExcel listener、缓存或显示格式语义，可被 Workbook
//! 读取、Event Mode 读取和 RoundTrip 模板处理共同使用。

use std::collections::HashMap;
use std::io::{BufReader, Read, Seek};

use easyexcel_io::{Error, Result};
use quick_xml::Reader;
use quick_xml::events::Event;
use zip::ZipArchive;

/// 仅包含包内关系的映射：`Id -> (Target, Type)`。
pub type Relationships = HashMap<String, (String, String)>;

/// 包含外部标记的关系映射：`Id -> (Target, Type, External)`。
pub type RawRelationships = HashMap<String, (String, String, bool)>;

/// 建立大小写不敏感的 ZIP part 路径缓存。
#[must_use]
pub fn path_cache<R: Read + Seek>(archive: &ZipArchive<R>) -> HashMap<String, String> {
    let mut paths = HashMap::with_capacity(archive.len());
    for name in archive.file_names() {
        paths.insert(name.to_ascii_lowercase(), name.to_owned());
    }
    paths
}

/// 按大小写不敏感缓存解析实际 ZIP part 名称。
#[must_use]
pub fn cached_path<'a>(cache: &'a HashMap<String, String>, path: &'a str) -> &'a str {
    cache
        .get(&path.to_ascii_lowercase())
        .map_or(path, String::as_str)
}

/// 从 ZIP 包读取内部关系，过滤 `TargetMode="External"`。
pub fn read_relationships<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    cache: &HashMap<String, String>,
    path: &str,
) -> Result<Relationships> {
    Ok(read_raw_relationships(archive, cache, path)?
        .into_iter()
        .filter_map(|(id, (target, relationship_type, external))| {
            (!external).then_some((id, (target, relationship_type)))
        })
        .collect())
}

/// 从 ZIP 包读取完整关系表。
pub fn read_raw_relationships<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    cache: &HashMap<String, String>,
    path: &str,
) -> Result<RawRelationships> {
    let actual = cached_path(cache, path);
    let file = archive.by_name(actual).map_err(Error::from)?;
    let mut reader = Reader::from_reader(BufReader::new(file));
    reader.config_mut().expand_empty_elements = true;
    let mut relationships = HashMap::new();
    let mut buffer = Vec::with_capacity(256);
    loop {
        buffer.clear();
        match reader.read_event_into(&mut buffer)? {
            Event::Start(element) if element.local_name().as_ref() == b"Relationship" => {
                let mut id = None;
                let mut target = String::new();
                let mut relationship_type = String::new();
                let mut external = false;
                for attribute in element.attributes().with_checks(false) {
                    let attribute = attribute.map_err(|error| Error::Xml(error.to_string()))?;
                    let local_name = attribute.key.local_name();
                    let key = String::from_utf8_lossy(local_name.as_ref());
                    let value = attribute
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Implicit1_0,
                            reader.decoder(),
                        )
                        .map_err(Error::from)?
                        .into_owned();
                    match key.as_ref() {
                        "Id" => id = Some(value),
                        "Target" => target = value,
                        "Type" => relationship_type = value,
                        "TargetMode" => external = value == "External",
                        _ => {}
                    }
                }
                if let Some(id) = id {
                    relationships.insert(id, (target, relationship_type, external));
                }
            }
            Event::End(element) if element.local_name().as_ref() == b"Relationships" => break,
            Event::Eof => {
                return Err(Error::Xml(format!("unexpected end of XML in {path}")));
            }
            _ => {}
        }
    }
    Ok(relationships)
}

/// 返回某个 part 对应的 `.rels` part 路径。
#[must_use]
pub fn relationship_part_name(path: &str) -> String {
    path.rsplit_once('/').map_or_else(
        || format!("_rels/{path}.rels"),
        |(directory, file)| format!("{directory}/_rels/{file}.rels"),
    )
}

/// 按 OOXML OPC 规则解析关系目标。
pub fn resolve_target(base_part: &str, target: &str) -> Result<String> {
    let candidate = if let Some(absolute) = target.strip_prefix('/') {
        absolute.to_owned()
    } else if let Some((directory, _)) = base_part.rsplit_once('/') {
        format!("{directory}/{target}")
    } else {
        target.to_owned()
    };
    normalize_path(&candidate)
}

/// 规范化包内路径并阻止 `..` 逃逸 ZIP 根目录。
pub fn normalize_path(path: &str) -> Result<String> {
    let mut components = Vec::new();
    for component in path.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if components.pop().is_none() {
                    return Err(Error::Xlsx(format!(
                        "OOXML relationship escapes package root: {path}"
                    )));
                }
            }
            value => components.push(value),
        }
    }
    if components.is_empty() {
        return Err(Error::Xlsx("empty OOXML relationship target".to_owned()));
    }
    Ok(components.join("/"))
}
