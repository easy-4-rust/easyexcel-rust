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

#[cfg(test)]
mod tests {
    use super::*;

    /// 最小 styles.xml 模板
    fn minimal_styles() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<fonts count="1"><font><sz val="11"/></font></fonts>
<fills count="2"><fill><patternFill patternType="none"/></fill><fill><patternFill patternType="gray125"/></fill></fills>
<borders count="1"><border/></borders>
<cellStyleXfs count="1"><xf/></cellStyleXfs>
<cellXfs count="2"><xf fontId="0" fillId="0" borderId="0"/><xf fontId="0" fillId="0" borderId="0" applyFont="1"/></cellXfs>
</styleSheet>"#.to_owned()
    }

    /// 编译源 styles.xml
    fn source_styles() -> String {
        r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<fonts count="1"><font><sz val="12"/></font></fonts>
<fills count="1"><fill><patternFill patternType="solid"><fgColor rgb="FFFF0000"/></patternFill></fill></fills>
<borders count="1"><border/></borders>
<cellStyleXfs count="1"><xf/></cellStyleXfs>
<cellXfs count="2"><xf fontId="0" fillId="0" borderId="0"/><xf fontId="0" fillId="0" borderId="0" applyFont="1"/></cellXfs>
</styleSheet>"#.to_owned()
    }

    // ── merge_compiled_styles 覆盖 ────────────────────────────────────

    #[test]
    fn merge_compiled_styles_empty_source_indexes() {
        let dest = minimal_styles();
        let source = source_styles();
        let (updated, mapped) = merge_compiled_styles(&dest, &source, &[]).unwrap();
        assert!(mapped.is_empty());
        assert!(!updated.is_empty());
    }

    #[test]
    fn merge_compiled_styles_maps_first_index() {
        let dest = minimal_styles();
        let source = source_styles();
        let (updated, mapped) = merge_compiled_styles(&dest, &source, &[1]).unwrap();
        assert_eq!(mapped.len(), 1);
        assert!(!updated.is_empty());
    }

    #[test]
    fn merge_compiled_styles_deduplicates_same_source_index() {
        let dest = minimal_styles();
        let source = source_styles();
        let (updated, mapped) = merge_compiled_styles(&dest, &source, &[0, 0]).unwrap();
        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0], mapped[1]); // 相同源索引应映射到相同目标
        assert!(!updated.is_empty());
    }

    #[test]
    fn merge_compiled_styles_error_for_out_of_range_index() {
        let dest = minimal_styles();
        let source = source_styles();
        let result = merge_compiled_styles(&dest, &source, &[999]);
        assert!(result.is_err());
    }

    // ── merge_compiled_styles_onto 覆盖 ───────────────────────────────

    #[test]
    fn merge_compiled_styles_onto_empty_returns_empty() {
        let dest = minimal_styles();
        let source = source_styles();
        let (_updated, mapped) = merge_compiled_styles_onto(&dest, &source, &[], &[]).unwrap();
        assert!(mapped.is_empty());
    }

    #[test]
    fn merge_compiled_styles_onto_error_for_count_mismatch() {
        let dest = minimal_styles();
        let source = source_styles();
        let result = merge_compiled_styles_onto(&dest, &source, &[0], &[0, 1]);
        assert!(result.is_err());
    }

    #[test]
    fn merge_compiled_styles_onto_maps_with_base() {
        let dest = minimal_styles();
        let source = source_styles();
        let (updated, mapped) = merge_compiled_styles_onto(&dest, &source, &[0], &[0]).unwrap();
        assert_eq!(mapped.len(), 1);
        assert!(!updated.is_empty());
    }

    #[test]
    fn merge_compiled_styles_onto_error_for_out_of_range_source() {
        let dest = minimal_styles();
        let source = source_styles();
        let result = merge_compiled_styles_onto(&dest, &source, &[999], &[0]);
        assert!(result.is_err());
    }

    #[test]
    fn merge_compiled_styles_onto_error_for_out_of_range_base() {
        let dest = minimal_styles();
        let source = source_styles();
        let result = merge_compiled_styles_onto(&dest, &source, &[0], &[999]);
        assert!(result.is_err());
    }

    // ── copy_alignment 覆盖 ──────────────────────────────────────────

    #[test]
    fn copy_alignment_replaces_existing() {
        let base = r#"<xf><alignment horizontal="center"/></xf>"#;
        let target = r#"<xf><alignment horizontal="left"/></xf>"#;
        let result = copy_alignment(base, target);
        assert!(result.contains("center"));
    }

    #[test]
    fn copy_alignment_inserts_into_self_closing() {
        let base = r#"<xf><alignment horizontal="right"/></xf>"#;
        let target = r#"<xf/>"#;
        let result = copy_alignment(base, target);
        assert!(result.contains("right"));
    }

    #[test]
    fn copy_alignment_appends_before_close() {
        let base = r#"<xf><alignment horizontal="center"/></xf>"#;
        let target = r#"<xf fontId="0"></xf>"#;
        let result = copy_alignment(base, target);
        assert!(result.contains("center"));
    }

    #[test]
    fn copy_alignment_no_alignment_in_base() {
        let base = r#"<xf/>"#;
        let target = r#"<xf fontId="0"/>"#;
        let result = copy_alignment(base, target);
        assert_eq!(result, target);
    }

    // ── extract_elements 覆盖 ─────────────────────────────────────────

    #[test]
    fn extract_elements_finds_multiple() {
        let xml = "<fonts><font>A</font><font>B</font></fonts>";
        let elems = extract_elements(xml, "font");
        assert_eq!(elems.len(), 2);
    }

    #[test]
    fn extract_elements_handles_self_closing_and_closed() {
        // 混合自闭合和闭合元素
        let xml = "<collection><entry attr=\"1\"/><entry>text</entry></collection>";
        let elems = extract_elements(xml, "entry");
        assert_eq!(elems.len(), 2);
    }

    #[test]
    fn extract_elements_empty_for_missing() {
        let elems = extract_elements("<root/>", "child");
        assert!(elems.is_empty());
    }

    // ── set_count_attribute 覆盖 ──────────────────────────────────────

    #[test]
    fn set_count_attribute_updates_existing() {
        let mut xml = r#"<fonts count="1">"#.to_owned();
        set_count_attribute(&mut xml, 5).unwrap();
        assert!(xml.contains("count=\"5\""));
    }

    #[test]
    fn set_count_attribute_inserts_when_missing() {
        let mut xml = "<fonts>".to_owned();
        set_count_attribute(&mut xml, 3).unwrap();
        assert!(xml.contains("count=\"3\""));
    }

    // ── replace_attribute 覆盖 ────────────────────────────────────────

    #[test]
    fn replace_attribute_updates_value() {
        let mut xml = r#"<xf fontId="0" fillId="1"/>"#.to_owned();
        replace_attribute(&mut xml, "fontId", "2").unwrap();
        assert!(xml.contains("fontId=\"2\""));
    }

    #[test]
    fn replace_attribute_error_for_missing() {
        let mut xml = "<xf/>".to_owned();
        let result = replace_attribute(&mut xml, "fontId", "2");
        assert!(result.is_err());
    }

    #[test]
    fn replace_attribute_error_for_unterminated() {
        let mut xml = r#"<xf fontId="0"#.to_owned();
        let result = replace_attribute(&mut xml, "fontId", "2");
        assert!(result.is_err());
    }

    // ── remap_number_format 覆盖 ──────────────────────────────────────

    #[test]
    fn remap_number_format_skips_builtin_ids() {
        let mut dest = minimal_styles();
        let source = source_styles();
        let mut xf = r#"<xf numFmtId="0"/>"#.to_owned();
        remap_number_format(&mut dest, &source, &mut xf).unwrap();
        assert!(xf.contains("numFmtId=\"0\""));
    }

    #[test]
    fn remap_number_format_imports_custom_format() {
        let dest = minimal_styles();
        let source = r##"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
<numFmts count="1"><numFmt numFmtId="164" formatCode="#,##0.00"/></numFmts>
<fonts count="1"><font/></fonts>
<fills count="1"><fill/></fills>
<borders count="1"><border/></borders>
<cellStyleXfs count="1"><xf/></cellStyleXfs>
<cellXfs count="1"><xf numFmtId="164"/></cellXfs>
</styleSheet>"##;
        let mut xf = r#"<xf numFmtId="164"/>"#.to_owned();
        remap_number_format(&mut String::from(dest), source, &mut xf).unwrap();
        // numFmtId 应被重映射
        assert!(xf.contains("numFmtId="));
    }

    // ── collection_elements 覆盖 ──────────────────────────────────────

    #[test]
    fn collection_elements_finds_children() {
        let xml =
            r#"<styleSheet><fonts count="1"><font><sz val="11"/></font></fonts></styleSheet>"#;
        let elems = collection_elements(xml, "fonts", "font").unwrap();
        assert_eq!(elems.len(), 1);
    }

    #[test]
    fn collection_elements_error_for_missing_collection() {
        let result = collection_elements("<styleSheet/>", "fonts", "font");
        assert!(result.is_err());
    }

    // ── optional_collection_elements 覆盖 ─────────────────────────────

    #[test]
    fn optional_collection_elements_returns_empty_for_missing() {
        let xml = "<styleSheet/>";
        let elems = optional_collection_elements(xml, "numFmts", "numFmt").unwrap();
        assert!(elems.is_empty());
    }

    #[test]
    fn optional_collection_elements_returns_elements_when_present() {
        let xml = r##"<styleSheet><numFmts count="1"><numFmt numFmtId="164" formatCode="#,##0"/></numFmts></styleSheet>"##;
        let elems = optional_collection_elements(xml, "numFmts", "numFmt").unwrap();
        assert_eq!(elems.len(), 1);
    }

    // ── collection_inner 覆盖 ─────────────────────────────────────────

    #[test]
    fn collection_inner_returns_none_for_missing() {
        let result = collection_inner("<styleSheet/>", "fonts").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn collection_inner_error_for_unclosed() {
        let result = collection_inner(r#"<styleSheet><fonts count="1">"#, "fonts");
        assert!(result.is_err());
    }

    // ── remap_index_attribute 覆盖 ────────────────────────────────────

    #[test]
    fn remap_index_attribute_maps_value() {
        let mut xml = r#"<xf fontId="0"/>"#.to_owned();
        let indexes = vec![5, 6, 7];
        remap_index_attribute(&mut xml, "fontId", &indexes).unwrap();
        assert!(xml.contains("fontId=\"5\""));
    }

    #[test]
    fn remap_index_attribute_skips_missing() {
        let mut xml = r#"<xf fillId="0"/>"#.to_owned();
        remap_index_attribute(&mut xml, "fontId", &[0]).unwrap();
        // fontId 不在元素中，应保持不变
        assert!(xml.contains("fillId=\"0\""));
    }

    #[test]
    fn remap_index_attribute_error_for_invalid_number() {
        let mut xml = r#"<xf fontId="abc"/>"#.to_owned();
        let result = remap_index_attribute(&mut xml, "fontId", &[0]);
        assert!(result.is_err());
    }

    #[test]
    fn remap_index_attribute_error_for_out_of_range() {
        let mut xml = r#"<xf fontId="5"/>"#.to_owned();
        let result = remap_index_attribute(&mut xml, "fontId", &[0, 1]);
        assert!(result.is_err());
    }

    // ── append_collection 覆盖 ────────────────────────────────────────

    #[test]
    fn append_collection_noop_when_empty() {
        let xml = minimal_styles();
        let result = append_collection(&xml, "fonts", "font", &[]).unwrap();
        assert_eq!(result, xml);
    }

    #[test]
    fn append_collection_appends_elements() {
        let xml = minimal_styles();
        let elems = vec!["<font><sz val=\"14\"/></font>".to_owned()];
        let result = append_collection(&xml, "fonts", "font", &elems).unwrap();
        assert!(result.contains("val=\"14\""));
    }

    // ── append_optional_collection 覆盖 ───────────────────────────────

    #[test]
    fn append_optional_collection_creates_when_missing() {
        let xml = minimal_styles();
        let elems = vec![r#"<numFmt numFmtId="200" formatCode="0.00"/>"#.to_owned()];
        let result =
            append_optional_collection(&xml, "numFmts", "numFmt", &elems, "<fonts").unwrap();
        assert!(result.contains("numFmtId=\"200\""));
    }

    #[test]
    fn append_optional_collection_error_for_missing_before() {
        let xml = "<styleSheet/>";
        let result =
            append_optional_collection(xml, "numFmts", "numFmt", &["test".to_owned()], "<fonts");
        assert!(result.is_err());
    }
}
