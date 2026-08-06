//! 中立 OOXML 模板占位符填充引擎。
//!
//! 该模块负责集合游标、标量替换、类型化单元格渲染和行追加；调用方只需
//! 将领域值转换为 [`TemplateCellValue`]，无需暴露 EasyExcel 门面类型。

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use easyexcel_io::{Error, Result};

use super::ooxml_package::OoxmlZipEntry;
use super::template_xml::{
    TemplateCellValue, all_cells, attribute_value, column_name, contains_unescaped, element_value,
    escape_xml, last_worksheet_row, parse_cell_reference, remove_attribute, replace_attribute,
    row_tag_with_reference, shared_string_values, shift_worksheet_rows_after, text_node_values,
    update_worksheet_dimension, upsert_collection_row, validate_collection_target,
    worksheet_max_row,
};

/// 集合填充方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TemplateFillDirection {
    /// 逐行向下填充。
    #[default]
    Vertical,
    /// 逐列向右填充。
    Horizontal,
}

/// 一行模板数据。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TemplateFillData {
    /// 占位符名称到中立值的映射。
    pub values: BTreeMap<String, TemplateCellValue>,
}

/// 一次集合填充请求。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TemplateCollectionFill {
    /// 可选集合前缀；`None` 对应 `{.field}`。
    pub name: Option<String>,
    /// 需要填充的数据行。
    pub rows: Vec<TemplateFillData>,
    /// 填充方向。
    pub direction: TemplateFillDirection,
    /// 是否在垂直填充时平移模板尾部行。
    pub force_new_row: bool,
    /// 是否保留模板单元格样式。
    pub auto_style: bool,
    /// 同一工作表中的调用顺序。
    pub order: usize,
}

#[derive(Debug, Clone)]
struct CollectionTemplateCell {
    row: usize,
    column: usize,
    row_tag: String,
    cell: String,
}

#[derive(Debug)]
struct CollectionFillCursor {
    templates: Vec<CollectionTemplateCell>,
    last_indices: Vec<Option<usize>>,
    initialized: bool,
}

/// 在指定 worksheet part 中依次执行集合填充。
#[allow(clippy::too_many_lines)]
pub fn replace_collection_fills_in_sheet(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    fills: &[TemplateCollectionFill],
) -> Result<()> {
    if fills.is_empty() {
        return Ok(());
    }
    let shared_strings = shared_strings(entries);
    let entry = entries
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(worksheet))
        .ok_or_else(|| Error::Xlsx(format!("worksheet part {worksheet:?} is missing")))?;
    let mut xml = std::str::from_utf8(&entry.bytes)
        .map_err(|error| Error::Xlsx(error.to_string()))?
        .to_owned();
    let mut cursors: BTreeMap<Option<String>, CollectionFillCursor> = BTreeMap::new();

    for fill in fills {
        let key = fill.name.clone();
        if !cursors.contains_key(&key) {
            let templates = collection_template_cells(&xml, fill.name.as_deref(), &shared_strings);
            let last_indices = vec![None; templates.len()];
            cursors.insert(
                key.clone(),
                CollectionFillCursor {
                    templates,
                    last_indices,
                    initialized: false,
                },
            );
        }
        if fill.direction == TemplateFillDirection::Vertical && fill.force_new_row {
            shift_following_rows_for_fill(&mut xml, &mut cursors, &key, fill.rows.len());
        }

        let cursor = cursors
            .get_mut(&key)
            .expect("collection cursor was initialized");
        for data in &fill.rows {
            for index in 0..cursor.templates.len() {
                let template = cursor.templates[index].clone();
                let (target_row, target_column, last_index) = match fill.direction {
                    TemplateFillDirection::Vertical => {
                        let row = cursor.last_indices[index]
                            .map_or(template.row, |last| last.saturating_add(1));
                        (row, template.column, row)
                    }
                    TemplateFillDirection::Horizontal => {
                        let column = cursor.last_indices[index]
                            .map_or(template.column, |last| last.saturating_add(1));
                        (template.row, column, column)
                    }
                };
                validate_collection_target(target_row, target_column)?;
                let cell = positioned_collection_cell(
                    &template.cell,
                    data,
                    fill.name.as_deref(),
                    &shared_strings,
                    fill.auto_style,
                    target_row,
                    target_column,
                );
                let row_tag =
                    replace_attribute(&template.row_tag, "r", &(target_row + 1).to_string());
                xml =
                    upsert_collection_row(&xml, &format!("{row_tag}{cell}</row>"), target_row + 1);
                cursor.last_indices[index] = Some(last_index);
            }
            cursor.initialized = true;
        }
    }
    entry.bytes = update_worksheet_dimension(&xml).into_bytes();
    Ok(())
}

/// 在指定 worksheet part 中替换标量占位符。
pub fn replace_scalar_cells_in_sheet(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    data: &TemplateFillData,
) -> Result<()> {
    replace_scalar_cells_matching(entries, Some(worksheet), data)
}

/// 在全部 worksheet part 中替换标量占位符。
pub fn replace_scalar_cells(entries: &mut [OoxmlZipEntry], data: &TemplateFillData) -> Result<()> {
    replace_scalar_cells_matching(entries, None, data)
}

/// 在指定 worksheet part 末尾追加普通行。
pub fn append_rows_to_sheet(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    rows: &[Vec<TemplateCellValue>],
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let entry = entries
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(worksheet))
        .ok_or_else(|| Error::Xlsx(format!("template does not contain {worksheet}")))?;
    let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
        .map_err(|error| Error::Xlsx(error.to_string()))?;
    entry.bytes = append_rows_to_xml(&xml, rows)?.into_bytes();
    Ok(())
}

/// 在工作表 XML 的 `sheetData` 末尾追加连续行。
pub fn append_rows_to_xml(xml: &str, rows: &[Vec<TemplateCellValue>]) -> Result<String> {
    let sheet_data_end = xml
        .find("</sheetData>")
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    let next_row = worksheet_max_row(&xml[..sheet_data_end]).saturating_add(1);
    let mut appended = String::new();
    for (row_offset, values) in rows.iter().enumerate() {
        let row_index = next_row + row_offset;
        let _ = write!(appended, "<row r=\"{row_index}\">");
        for (column, value) in values.iter().enumerate() {
            let reference = format!("{}{row_index}", column_name(column + 1));
            appended.push_str(&render_typed_cell(
                &format!("<c r=\"{reference}\"></c>"),
                value,
                true,
            ));
        }
        appended.push_str("</row>");
    }
    let expanded = format!(
        "{}{}{}",
        &xml[..sheet_data_end],
        appended,
        &xml[sheet_data_end..]
    );
    Ok(update_worksheet_dimension(&expanded))
}

/// 在单个工作表 XML 中替换标量占位符。
#[must_use]
pub fn replace_scalar_cells_in_xml(
    xml: &str,
    data: &TemplateFillData,
    shared_strings: &[String],
) -> String {
    let mut output = String::with_capacity(xml.len());
    let mut offset = 0;
    while let Some((start, end)) = find_next_cell(xml, offset) {
        let cell = &xml[start..end];
        output.push_str(&xml[offset..start]);
        let replacement = cell_value(cell, shared_strings).map_or_else(
            || cell.to_owned(),
            |placeholder| {
                if let Some(value) = exact_scalar_value(&placeholder, data) {
                    return render_typed_cell(cell, value, true);
                }
                let filled = replace_template_values(&placeholder, &data.values, None, true, false);
                if filled == placeholder {
                    cell.to_owned()
                } else {
                    render_typed_cell(cell, &TemplateCellValue::Text(filled), true)
                }
            },
        );
        output.push_str(&replacement);
        offset = end;
    }
    output.push_str(&xml[offset..]);
    output
}

/// 渲染保留原坐标与可选样式的类型化单元格。
#[must_use]
pub fn render_typed_cell(cell: &str, value: &TemplateCellValue, auto_style: bool) -> String {
    let Some(tag_end) = cell.find('>') else {
        return cell.to_owned();
    };
    let mut start = cell[..=tag_end].to_owned();
    if !auto_style {
        start = remove_attribute(&start, "s");
    }
    start = remove_attribute(&start, "t");
    match value {
        TemplateCellValue::Empty => format!("{start}</c>"),
        TemplateCellValue::Text(value) => {
            insert_cell_type(&mut start, "inlineStr");
            format!("{start}<is><t>{}</t></is></c>", escape_xml(value))
        }
        TemplateCellValue::Bool(value) => {
            insert_cell_type(&mut start, "b");
            format!("{start}<v>{}</v></c>", u8::from(*value))
        }
        TemplateCellValue::Number(value) => format!("{start}<v>{value}</v></c>"),
        TemplateCellValue::Date(value) => {
            insert_cell_type(&mut start, "d");
            format!("{start}<v>{}</v></c>", escape_xml(value))
        }
        TemplateCellValue::Formula(value) => {
            format!("{start}<f>{}</f><v></v></c>", escape_xml(value))
        }
        TemplateCellValue::Error(value) => {
            insert_cell_type(&mut start, "e");
            format!("{start}<v>{}</v></c>", escape_xml(value))
        }
    }
}

fn shift_following_rows_for_fill(
    xml: &mut String,
    cursors: &mut BTreeMap<Option<String>, CollectionFillCursor>,
    key: &Option<String>,
    row_count: usize,
) {
    let Some(cursor) = cursors.get(key) else {
        return;
    };
    if cursor.templates.is_empty() {
        return;
    }
    let maximum = cursor
        .templates
        .iter()
        .zip(&cursor.last_indices)
        .map(|(template, last)| last.unwrap_or(template.row))
        .max()
        .unwrap_or(0);
    let shift = row_count.saturating_sub(usize::from(!cursor.initialized));
    if shift == 0 || maximum >= last_worksheet_row(xml).unwrap_or(maximum) {
        return;
    }
    *xml = shift_worksheet_rows_after(xml, maximum, shift);
    for cached in cursors.values_mut() {
        for template in &mut cached.templates {
            if template.row > maximum {
                template.row = template.row.saturating_add(shift);
                template.row_tag =
                    replace_attribute(&template.row_tag, "r", &(template.row + 1).to_string());
            }
        }
    }
}

fn collection_template_cells(
    xml: &str,
    name: Option<&str>,
    shared_strings: &[String],
) -> Vec<CollectionTemplateCell> {
    let mut templates = Vec::new();
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find("</row>") else {
            break;
        };
        let end = start + relative_end + 6;
        let row_xml = &xml[start..end];
        let Some(tag_end) = row_xml.find('>') else {
            break;
        };
        for (_, _, cell) in collection_cells(row_xml, name, shared_strings) {
            let Some(reference) = attribute_value(cell, "r") else {
                continue;
            };
            let Some((column, row)) = parse_cell_reference(reference) else {
                continue;
            };
            templates.push(CollectionTemplateCell {
                row: row - 1,
                column: column - 1,
                row_tag: row_tag_with_reference(&row_xml[..=tag_end], row),
                cell: cell.to_owned(),
            });
        }
        offset = end;
    }
    templates
}

fn collection_cells<'a>(
    row: &'a str,
    name: Option<&str>,
    shared_strings: &[String],
) -> Vec<(usize, usize, &'a str)> {
    all_cells(row)
        .into_iter()
        .filter(|(_, _, cell)| {
            cell_value(cell, shared_strings)
                .is_some_and(|value| contains_collection_marker(&value, name))
        })
        .collect()
}

fn positioned_collection_cell(
    template_cell: &str,
    data: &TemplateFillData,
    prefix: Option<&str>,
    shared_strings: &[String],
    auto_style: bool,
    row: usize,
    column: usize,
) -> String {
    let cell = fill_cell(template_cell, data, prefix, shared_strings, auto_style);
    replace_attribute(
        &cell,
        "r",
        &format!("{}{}", column_name(column + 1), row + 1),
    )
}

fn fill_cell(
    cell: &str,
    data: &TemplateFillData,
    prefix: Option<&str>,
    shared_strings: &[String],
    auto_style: bool,
) -> String {
    let Some(tag_end) = cell.find('>') else {
        return cell.to_owned();
    };
    let Some(value) = cell_value(cell, shared_strings) else {
        return cell.to_owned();
    };
    if let Some(typed_value) = exact_collection_value(&value, data, prefix) {
        return render_typed_cell(cell, typed_value, auto_style);
    }
    let filled = replace_template_values(&value, &data.values, prefix, false, false);
    if filled == value {
        return cell.to_owned();
    }
    let mut start = cell[..=tag_end].replace(" t=\"s\"", "");
    if !auto_style {
        start = remove_attribute(&start, "s");
    }
    if start.contains(" t=\"") {
        start = replace_attribute(&start, "t", "inlineStr");
    } else {
        start.insert_str(start.len() - 1, " t=\"inlineStr\"");
    }
    format!("{start}<is><t>{}</t></is></c>", escape_xml(&filled))
}

fn exact_collection_value<'a>(
    placeholder: &str,
    data: &'a TemplateFillData,
    prefix: Option<&str>,
) -> Option<&'a TemplateCellValue> {
    let variable = placeholder.strip_prefix('{')?.strip_suffix('}')?;
    let key = match prefix {
        Some(prefix) => variable.strip_prefix(prefix)?.strip_prefix('.')?,
        None => variable.strip_prefix('.')?,
    };
    data.values.get(key)
}

fn exact_scalar_value<'a>(
    placeholder: &str,
    data: &'a TemplateFillData,
) -> Option<&'a TemplateCellValue> {
    let key = placeholder.strip_prefix('{')?.strip_suffix('}')?;
    (!key.starts_with('.') && !key.ends_with('.'))
        .then(|| data.values.get(key))
        .flatten()
}

fn cell_value(cell: &str, shared_strings: &[String]) -> Option<String> {
    if attribute_value(cell, "t") == Some("s") {
        let index = element_value(cell, "v")?.parse::<usize>().ok()?;
        return shared_strings.get(index).cloned();
    }
    let value = text_node_values(cell);
    (!value.is_empty()).then_some(value)
}

fn contains_collection_marker(value: &str, name: Option<&str>) -> bool {
    let prefix = name.map_or(".".to_owned(), |name| format!("{name}."));
    contains_unescaped(value, &format!("{{{prefix}"))
}

fn replace_scalar_cells_matching(
    entries: &mut [OoxmlZipEntry],
    worksheet: Option<&str>,
    data: &TemplateFillData,
) -> Result<()> {
    let shared_strings = shared_strings(entries);
    for entry in entries.iter_mut().filter(|entry| {
        !entry.is_dir
            && worksheet.map_or_else(
                || entry.name.starts_with("xl/worksheets/"),
                |worksheet| entry.name.eq_ignore_ascii_case(worksheet),
            )
            && Path::new(&entry.name)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
    }) {
        let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        entry.bytes = replace_scalar_cells_in_xml(&xml, data, &shared_strings).into_bytes();
    }
    Ok(())
}

fn shared_strings(entries: &[OoxmlZipEntry]) -> Vec<String> {
    entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case("xl/sharedStrings.xml"))
        .and_then(|entry| std::str::from_utf8(&entry.bytes).ok())
        .map(shared_string_values)
        .unwrap_or_default()
}

fn insert_cell_type(start: &mut String, cell_type: &str) {
    start.insert_str(start.len() - 1, &format!(" t=\"{cell_type}\""));
}

fn replace_template_values(
    input: &str,
    values: &BTreeMap<String, TemplateCellValue>,
    collection_prefix: Option<&str>,
    scalar_values: bool,
    escape_values: bool,
) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\'
            && bytes
                .get(index + 1)
                .is_some_and(|next| matches!(next, b'{' | b'}'))
        {
            output.push(char::from(bytes[index + 1]));
            index += 2;
            continue;
        }
        if bytes[index] == b'{'
            && let Some(relative_end) = input[index + 1..].find('}')
        {
            let end = index + relative_end + 1;
            let placeholder = &input[index + 1..end];
            let key = if scalar_values {
                Some(placeholder)
            } else {
                match collection_prefix {
                    Some(prefix) => placeholder
                        .strip_prefix(prefix)
                        .and_then(|value| value.strip_prefix('.')),
                    None => placeholder.strip_prefix('.'),
                }
            };
            if let Some(value) = key.and_then(|key| values.get(key)) {
                let value = value.as_text();
                if escape_values {
                    output.push_str(&escape_xml(&value));
                } else {
                    output.push_str(&value);
                }
                index = end + 1;
                continue;
            }
        }
        let character = input[index..]
            .chars()
            .next()
            .expect("index always points to a character boundary");
        output.push(character);
        index += character.len_utf8();
    }
    output
}

fn find_next_cell(xml: &str, offset: usize) -> Option<(usize, usize)> {
    let relative = xml.get(offset..)?.find("<c")?;
    let start = offset + relative;
    let after = xml.as_bytes().get(start + 2).copied()?;
    if after.is_ascii_alphanumeric() {
        return find_next_cell(xml, start + 2);
    }
    let tag_end = start + xml[start..].find('>')?;
    if xml[..=tag_end].ends_with("/>") {
        return Some((start, tag_end + 1));
    }
    let end = tag_end + 1 + xml[tag_end + 1..].find("</c>")? + 4;
    Some((start, end))
}
