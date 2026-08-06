//! XLSX 模板 styles.xml 组件合并与索引重映射。

use easyexcel_io::{Error, Result};

use super::template_xml::attribute_value;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将编译工作簿中的字体、填充、边框、数字格式和 cell XF 合并到模板 styles.xml。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn merge_compiled_styles(
    destination: &str,
    source: &str,
    source_indexes: &[usize],
) -> Result<(String, Vec<u32>)> {
    let source_fonts = collection_elements(source, "fonts", "font")?;
    let source_fills = collection_elements(source, "fills", "fill")?;
    let source_borders = collection_elements(source, "borders", "border")?;
    let source_xfs = collection_elements(source, "cellXfs", "xf")?;
    let (mut updated, font_indexes) =
        merge_component_collection(destination, "fonts", "font", &source_fonts)?;
    let (next, fill_indexes) =
        merge_component_collection(&updated, "fills", "fill", &source_fills)?;
    updated = next;
    let (next, border_indexes) =
        merge_component_collection(&updated, "borders", "border", &source_borders)?;
    updated = next;

    let mut imported = std::collections::HashMap::new();
    let mut appended_xfs = Vec::new();
    let destination_xfs = collection_elements(&updated, "cellXfs", "xf")?;
    let mut mapped = Vec::with_capacity(source_indexes.len());
    for source_index in source_indexes {
        if let Some(destination_index) = imported.get(source_index).copied() {
            mapped.push(destination_index);
            continue;
        }
        let source_xf = source_xfs.get(*source_index).ok_or_else(|| {
            Error::Xlsx(format!(
                "compiled style index {source_index} is out of range"
            ))
        })?;
        let mut xf = source_xf.clone();
        remap_index_attribute(&mut xf, "fontId", &font_indexes)?;
        remap_index_attribute(&mut xf, "fillId", &fill_indexes)?;
        remap_index_attribute(&mut xf, "borderId", &border_indexes)?;
        remap_number_format(&mut updated, source, &mut xf)?;
        let destination_index = destination_xfs
            .iter()
            .chain(appended_xfs.iter())
            .position(|existing| existing == &xf)
            .map_or_else(
                || {
                    let index = u32::try_from(destination_xfs.len() + appended_xfs.len())
                        .unwrap_or(u32::MAX);
                    appended_xfs.push(xf);
                    index
                },
                |index| u32::try_from(index).unwrap_or(u32::MAX),
            );
        if destination_index == u32::MAX {
            return Err(Error::Xlsx("template cell style index overflow".to_owned()));
        }
        imported.insert(*source_index, destination_index);
        mapped.push(destination_index);
    }
    updated = append_collection(&updated, "cellXfs", "xf", &appended_xfs)?;
    Ok((updated, mapped))
}

/// 将编译样式叠加到模板已有 XF，未显式设置的格式属性保持模板原值。
///
/// # Errors
///
/// 当任一 styles.xml 结构无效或样式索引无法映射时返回错误。
pub fn merge_compiled_styles_onto(
    destination: &str,
    source: &str,
    source_indexes: &[usize],
    base_indexes: &[usize],
) -> Result<(String, Vec<u32>)> {
    if source_indexes.len() != base_indexes.len() {
        return Err(Error::Xlsx(
            "compiled style and base style counts differ".to_owned(),
        ));
    }
    let source_fonts = collection_elements(source, "fonts", "font")?;
    let source_fills = collection_elements(source, "fills", "fill")?;
    let source_borders = collection_elements(source, "borders", "border")?;
    let source_xfs = collection_elements(source, "cellXfs", "xf")?;
    let (mut updated, font_indexes) =
        merge_component_collection(destination, "fonts", "font", &source_fonts)?;
    let (next, fill_indexes) =
        merge_component_collection(&updated, "fills", "fill", &source_fills)?;
    updated = next;
    let (next, border_indexes) =
        merge_component_collection(&updated, "borders", "border", &source_borders)?;
    updated = next;
    let destination_xfs = collection_elements(&updated, "cellXfs", "xf")?;
    let mut appended_xfs = Vec::new();
    let mut mapped = Vec::with_capacity(source_indexes.len());
    for (source_index, base_index) in source_indexes.iter().zip(base_indexes) {
        let source_xf = source_xfs.get(*source_index).ok_or_else(|| {
            Error::Xlsx(format!(
                "compiled style index {source_index} is out of range"
            ))
        })?;
        let base_xf = destination_xfs.get(*base_index).ok_or_else(|| {
            Error::Xlsx(format!(
                "template base style index {base_index} is out of range"
            ))
        })?;
        let mut xf = source_xf.clone();
        remap_index_attribute(&mut xf, "fontId", &font_indexes)?;
        remap_index_attribute(&mut xf, "fillId", &fill_indexes)?;
        remap_index_attribute(&mut xf, "borderId", &border_indexes)?;
        remap_number_format(&mut updated, source, &mut xf)?;
        for (apply, attribute) in [
            ("applyFont", "fontId"),
            ("applyFill", "fillId"),
            ("applyBorder", "borderId"),
            ("applyNumberFormat", "numFmtId"),
        ] {
            if attribute_value(&xf, apply) != Some("1")
                && let Some(base_value) = attribute_value(base_xf, attribute)
            {
                replace_attribute(&mut xf, attribute, base_value)?;
            }
        }
        if attribute_value(&xf, "applyAlignment") != Some("1") {
            xf = copy_alignment(base_xf, &xf);
        }
        let destination_index = destination_xfs
            .iter()
            .chain(appended_xfs.iter())
            .position(|existing| existing == &xf)
            .map_or_else(
                || {
                    let index = u32::try_from(destination_xfs.len() + appended_xfs.len())
                        .unwrap_or(u32::MAX);
                    appended_xfs.push(xf);
                    index
                },
                |index| u32::try_from(index).unwrap_or(u32::MAX),
            );
        mapped.push(destination_index);
    }
    updated = append_collection(&updated, "cellXfs", "xf", &appended_xfs)?;
    Ok((updated, mapped))
}

fn copy_alignment(base: &str, target: &str) -> String {
    let Some(alignment) = extract_elements(base, "alignment").into_iter().next() else {
        return target.to_owned();
    };
    if let Some(current) = extract_elements(target, "alignment").into_iter().next() {
        return target.replacen(&current, &alignment, 1);
    }
    if let Some(prefix) = target.strip_suffix("/>") {
        return format!("{prefix}>{alignment}</xf>");
    }
    target.replacen("</xf>", &format!("{alignment}</xf>"), 1)
}

fn remap_index_attribute(xml: &mut String, name: &str, indexes: &[usize]) -> Result<()> {
    let Some(value) = attribute_value(xml, name) else {
        return Ok(());
    };
    let index = value
        .parse::<usize>()
        .map_err(|_| Error::Xlsx(format!("invalid {name} in compiled style")))?;
    let mapped = indexes.get(index).ok_or_else(|| {
        Error::Xlsx(format!(
            "compiled style {name} index {index} is out of range"
        ))
    })?;
    replace_attribute(xml, name, &mapped.to_string())
}

fn merge_component_collection(
    xml: &str,
    collection: &str,
    child: &str,
    source: &[String],
) -> Result<(String, Vec<usize>)> {
    let destination = collection_elements(xml, collection, child)?;
    let mut appended = Vec::new();
    let mut indexes = Vec::with_capacity(source.len());
    for component in source {
        let index = destination
            .iter()
            .chain(appended.iter())
            .position(|existing| existing == component)
            .unwrap_or_else(|| {
                let index = destination.len() + appended.len();
                appended.push(component.clone());
                index
            });
        indexes.push(index);
    }
    Ok((
        append_collection(xml, collection, child, &appended)?,
        indexes,
    ))
}

fn remap_number_format(destination: &mut String, source: &str, xf: &mut String) -> Result<()> {
    let Some(value) = attribute_value(xf, "numFmtId") else {
        return Ok(());
    };
    let source_id = value
        .parse::<u32>()
        .map_err(|_| Error::Xlsx("invalid numFmtId in compiled style".to_owned()))?;
    if source_id < 164 {
        return Ok(());
    }
    let source_formats = optional_collection_elements(source, "numFmts", "numFmt")?;
    let source_format = source_formats
        .iter()
        .find(|format| attribute_value(format, "numFmtId") == Some(value))
        .ok_or_else(|| Error::Xlsx(format!("compiled style is missing numFmtId {source_id}")))?;
    let code = attribute_value(source_format, "formatCode")
        .ok_or_else(|| Error::Xlsx("compiled numFmt has no formatCode".to_owned()))?;
    let destination_formats = optional_collection_elements(destination, "numFmts", "numFmt")?;
    if let Some(existing) = destination_formats
        .iter()
        .find(|format| attribute_value(format, "formatCode") == Some(code))
    {
        let id = attribute_value(existing, "numFmtId")
            .ok_or_else(|| Error::Xlsx("template numFmt has no id".to_owned()))?;
        return replace_attribute(xf, "numFmtId", id);
    }
    let next_id = destination_formats
        .iter()
        .filter_map(|format| attribute_value(format, "numFmtId")?.parse::<u32>().ok())
        .max()
        .unwrap_or(163)
        .saturating_add(1)
        .max(164);
    let mut imported = source_format.clone();
    replace_attribute(&mut imported, "numFmtId", &next_id.to_string())?;
    *destination =
        append_optional_collection(destination, "numFmts", "numFmt", &[imported], "<fonts")?;
    replace_attribute(xf, "numFmtId", &next_id.to_string())
}

fn collection_elements(xml: &str, collection: &str, child: &str) -> Result<Vec<String>> {
    let (inner, _) = collection_inner(xml, collection)?
        .ok_or_else(|| Error::Xlsx(format!("styles.xml is missing {collection}")))?;
    Ok(extract_elements(inner, child))
}

fn optional_collection_elements(xml: &str, collection: &str, child: &str) -> Result<Vec<String>> {
    Ok(collection_inner(xml, collection)?
        .map(|(inner, _)| extract_elements(inner, child))
        .unwrap_or_default())
}

// 语义敏感：返回 (标签区间, 行/列/子元素计数) 三元组以驱动模板改写循环，
// 拆 type 别名反而割裂阅读上下文，故豁免 type_complexity。
#[allow(clippy::type_complexity)]
fn collection_inner<'a>(
    xml: &'a str,
    collection: &str,
) -> Result<Option<(&'a str, (usize, usize, usize))>> {
    let marker = format!("<{collection}");
    let Some(start) = xml.find(&marker) else {
        return Ok(None);
    };
    let open_end = start
        + xml[start..]
            .find('>')
            .ok_or_else(|| Error::Xlsx(format!("malformed {collection} element")))?;
    let close_marker = format!("</{collection}>");
    let close = open_end
        + 1
        + xml[open_end + 1..]
            .find(&close_marker)
            .ok_or_else(|| Error::Xlsx(format!("malformed {collection} element")))?;
    Ok(Some((&xml[open_end + 1..close], (start, open_end, close))))
}

fn extract_elements(xml: &str, child: &str) -> Vec<String> {
    let marker = format!("<{child}");
    let close_marker = format!("</{child}>");
    let mut elements = Vec::new();
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find(&marker) {
        let start = offset + relative_start;
        let Some(relative_open_end) = xml[start..].find('>') else {
            break;
        };
        let open_end = start + relative_open_end;
        let end = if xml[..=open_end].ends_with("/>") {
            open_end + 1
        } else if let Some(relative_close) = xml[open_end + 1..].find(&close_marker) {
            open_end + 1 + relative_close + close_marker.len()
        } else {
            break;
        };
        elements.push(xml[start..end].to_owned());
        offset = end;
    }
    elements
}

fn append_collection(
    xml: &str,
    collection: &str,
    child: &str,
    elements: &[String],
) -> Result<String> {
    if elements.is_empty() {
        return Ok(xml.to_owned());
    }
    let (_, (start, open_end, close)) = collection_inner(xml, collection)?
        .ok_or_else(|| Error::Xlsx(format!("styles.xml is missing {collection}")))?;
    let current = extract_elements(&xml[open_end + 1..close], child).len();
    let mut opening = xml[start..=open_end].to_owned();
    set_count_attribute(&mut opening, current + elements.len())?;
    Ok(format!(
        "{}{}{}{}{}",
        &xml[..start],
        opening,
        &xml[open_end + 1..close],
        elements.concat(),
        &xml[close..]
    ))
}

fn append_optional_collection(
    xml: &str,
    collection: &str,
    child: &str,
    elements: &[String],
    before: &str,
) -> Result<String> {
    if collection_inner(xml, collection)?.is_some() {
        return append_collection(xml, collection, child, elements);
    }
    let insertion = xml
        .find(before)
        .ok_or_else(|| Error::Xlsx(format!("styles.xml is missing {before}")))?;
    Ok(format!(
        "{}<{} count=\"{}\">{}</{}>{}",
        &xml[..insertion],
        collection,
        elements.len(),
        elements.concat(),
        collection,
        &xml[insertion..]
    ))
}

fn set_count_attribute(opening: &mut String, count: usize) -> Result<()> {
    if attribute_value(opening, "count").is_some() {
        replace_attribute(opening, "count", &count.to_string())
    } else {
        let insertion = opening
            .find('>')
            .ok_or_else(|| Error::Xlsx("malformed style collection".to_owned()))?;
        opening.insert_str(insertion, &format!(" count=\"{count}\""));
        Ok(())
    }
}

fn replace_attribute(xml: &mut String, name: &str, replacement: &str) -> Result<()> {
    let marker = format!("{name}=\"");
    let start = xml
        .find(&marker)
        .ok_or_else(|| Error::Xlsx(format!("missing {name} attribute")))?
        + marker.len();
    let end = start
        + xml[start..]
            .find('"')
            .ok_or_else(|| Error::Xlsx(format!("unterminated {name} attribute")))?;
    xml.replace_range(start..end, replacement);
    Ok(())
}
