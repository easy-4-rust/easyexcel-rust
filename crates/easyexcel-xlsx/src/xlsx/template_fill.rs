//! 中立 OOXML 模板占位符填充引擎。
//!
//! 该模块负责集合游标、标量替换、类型化单元格渲染和行追加；调用方只需
//! 将领域值转换为 [`TemplateCellValue`]，无需暴露 `EasyExcel` 门面类型。

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

include!("template_fill/template_fill_direction.rs");
include!("template_fill/template_comment.rs");
include!("template_fill/template_comment_placement.rs");
include!("template_fill/template_hyperlink_type.rs");
include!("template_fill/template_hyperlink_coordinate.rs");
include!("template_fill/template_hyperlink.rs");
include!("template_fill/template_image.rs");
include!("template_fill/template_decoration.rs");
include!("template_fill/template_decoration_placement.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 一行模板数据。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TemplateFillData {
    /// 占位符名称到中立值的映射。
    pub values: BTreeMap<String, TemplateCellValue>,
}

include!("template_fill/template_collection_fill.rs");

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

/// 返回集合占位符各物理列的模板样式索引。
#[must_use]
pub fn collection_column_style_indexes(
    entries: &[OoxmlZipEntry],
    worksheet: &str,
) -> BTreeMap<usize, usize> {
    let shared_strings = shared_strings(entries);
    let Some(entry) = entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(worksheet))
    else {
        return BTreeMap::new();
    };
    let Ok(xml) = std::str::from_utf8(&entry.bytes) else {
        return BTreeMap::new();
    };
    collection_template_cells(xml, None, &shared_strings)
        .into_iter()
        .map(|cell| {
            let style = attribute_value(&cell.cell, "s")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(0);
            (cell.column, style)
        })
        .collect()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在指定 worksheet part 中依次执行集合填充。
#[allow(clippy::too_many_lines)]
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn replace_collection_fills_in_sheet(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    fills: &[TemplateCollectionFill],
) -> Result<()> {
    replace_collection_fills_in_sheet_with_decorations(entries, worksheet, fills).map(|_| ())
}

/// 在指定 worksheet 中执行集合填充，并返回批注最终物理坐标。
pub fn replace_collection_fills_in_sheet_with_comments(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    fills: &[TemplateCollectionFill],
) -> Result<Vec<TemplateCommentPlacement>> {
    replace_collection_fills_in_sheet_with_decorations(entries, worksheet, fills)
        .map(comment_placements)
}

/// 在指定 worksheet 中执行集合填充，并返回全部 package 层装饰的最终物理坐标。
pub fn replace_collection_fills_in_sheet_with_decorations(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    fills: &[TemplateCollectionFill],
) -> Result<Vec<TemplateDecorationPlacement>> {
    if fills.is_empty() {
        return Ok(Vec::new());
    }
    let mut decoration_placements: Vec<TemplateDecorationPlacement> = Vec::new();
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
            if let Some((maximum, shift)) =
                shift_following_rows_for_fill(&mut xml, &mut cursors, &key, fill.rows.len())
            {
                for placement in &mut decoration_placements {
                    if usize::try_from(placement.row).unwrap_or(usize::MAX) > maximum {
                        placement.row = placement
                            .row
                            .saturating_add(u32::try_from(shift).unwrap_or(u32::MAX));
                    }
                }
            }
        }

        let Some(cursor) = cursors.get_mut(&key) else {
            return Err(Error::Xlsx(
                "collection fill cursor initialization failed".to_owned(),
            ));
        };
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
                if let Some(placeholder) = cell_value(&template.cell, &shared_strings)
                    && let Some(value) = exact_collection_value(
                        &placeholder,
                        data,
                        fill.name.as_deref(),
                    )
                {
                    decoration_placements.extend(template_decoration_placements(
                        value,
                        target_row,
                        target_column,
                    ));
                }
                let cell = positioned_collection_cell(
                    &template.cell,
                    data,
                    fill.name.as_deref(),
                    &shared_strings,
                    fill.auto_style,
                    target_row,
                    target_column,
                    fill.column_styles.get(&target_column).copied(),
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
    Ok(decoration_placements)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在指定 worksheet part 中替换标量占位符。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn replace_scalar_cells_in_sheet(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    data: &TemplateFillData,
) -> Result<()> {
    replace_scalar_cells_in_sheet_with_decorations(entries, worksheet, data).map(|_| ())
}

/// 替换标量占位符，并返回精确批注值对应的最终物理坐标。
pub fn replace_scalar_cells_in_sheet_with_comments(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    data: &TemplateFillData,
) -> Result<Vec<TemplateCommentPlacement>> {
    replace_scalar_cells_in_sheet_with_decorations(entries, worksheet, data).map(comment_placements)
}

/// 替换标量占位符，并返回全部 package 层装饰的最终物理坐标。
pub fn replace_scalar_cells_in_sheet_with_decorations(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    data: &TemplateFillData,
) -> Result<Vec<TemplateDecorationPlacement>> {
    replace_scalar_cells_matching_with_decorations(entries, Some(worksheet), data)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在全部 worksheet part 中替换标量占位符。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn replace_scalar_cells(entries: &mut [OoxmlZipEntry], data: &TemplateFillData) -> Result<()> {
    replace_scalar_cells_matching_with_decorations(entries, None, data).map(|_| ())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在指定 worksheet part 末尾追加普通行。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn append_rows_to_sheet(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    rows: &[Vec<TemplateCellValue>],
) -> Result<()> {
    append_rows_to_sheet_with_decorations(entries, worksheet, rows).map(|_| ())
}

/// 追加普通行，并返回批注值对应的最终物理坐标。
pub fn append_rows_to_sheet_with_comments(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    rows: &[Vec<TemplateCellValue>],
) -> Result<Vec<TemplateCommentPlacement>> {
    append_rows_to_sheet_with_decorations(entries, worksheet, rows).map(comment_placements)
}

/// 追加普通行，并返回全部 package 层装饰的最终物理坐标。
pub fn append_rows_to_sheet_with_decorations(
    entries: &mut [OoxmlZipEntry],
    worksheet: &str,
    rows: &[Vec<TemplateCellValue>],
) -> Result<Vec<TemplateDecorationPlacement>> {
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    let entry = entries
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(worksheet))
        .ok_or_else(|| Error::Xlsx(format!("template does not contain {worksheet}")))?;
    let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
        .map_err(|error| Error::Xlsx(error.to_string()))?;
    let (xml, decorations) = append_rows_to_xml_with_decorations(&xml, rows)?;
    entry.bytes = xml.into_bytes();
    Ok(decorations)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在工作表 XML 的 `sheetData` 末尾追加连续行。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
pub fn append_rows_to_xml(xml: &str, rows: &[Vec<TemplateCellValue>]) -> Result<String> {
    append_rows_to_xml_with_decorations(xml, rows).map(|(xml, _)| xml)
}

fn append_rows_to_xml_with_decorations(
    xml: &str,
    rows: &[Vec<TemplateCellValue>],
) -> Result<(String, Vec<TemplateDecorationPlacement>)> {
    let sheet_data_end = xml
        .find("</sheetData>")
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    let next_row = worksheet_max_row(&xml[..sheet_data_end]).saturating_add(1);
    let mut appended = String::new();
    let mut decorations = Vec::new();
    for (row_offset, values) in rows.iter().enumerate() {
        let row_index = next_row + row_offset;
        let _ = write!(appended, "<row r=\"{row_index}\">");
        for (column, value) in values.iter().enumerate() {
            decorations.extend(template_decoration_placements(
                value,
                row_index.saturating_sub(1),
                column,
            ));
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
    Ok((update_worksheet_dimension(&expanded), decorations))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在单个工作表 XML 中替换标量占位符。
#[must_use]
pub fn replace_scalar_cells_in_xml(
    xml: &str,
    data: &TemplateFillData,
    shared_strings: &[String],
) -> String {
    replace_scalar_cells_in_xml_with_decorations(xml, data, shared_strings).0
}

fn replace_scalar_cells_in_xml_with_decorations(
    xml: &str,
    data: &TemplateFillData,
    shared_strings: &[String],
) -> (String, Vec<TemplateDecorationPlacement>) {
    let mut output = String::with_capacity(xml.len());
    let mut offset = 0;
    let mut decorations = Vec::new();
    while let Some((start, end)) = find_next_cell(xml, offset) {
        let cell = &xml[start..end];
        output.push_str(&xml[offset..start]);
        let replacement = if let Some(placeholder) = cell_value(cell, shared_strings) {
            if let Some(value) = exact_scalar_value(&placeholder, data) {
                if let Some(reference) = attribute_value(cell, "r")
                    && let Some((column, row)) = parse_cell_reference(reference)
                {
                    decorations.extend(template_decoration_placements(
                        value,
                        row.saturating_sub(1),
                        column.saturating_sub(1),
                    ));
                }
                render_typed_cell(cell, value, true)
            } else {
                let filled =
                    replace_template_values(&placeholder, &data.values, None, true, false);
                if filled == placeholder {
                    cell.to_owned()
                } else {
                    render_typed_cell(cell, &TemplateCellValue::Text(filled), true)
                }
            }
        } else {
            cell.to_owned()
        };
        output.push_str(&replacement);
        offset = end;
    }
    output.push_str(&xml[offset..]);
    (output, decorations)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 渲染保留原坐标与可选样式的类型化单元格。
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
        TemplateCellValue::RichText(value) => {
            insert_cell_type(&mut start, "inlineStr");
            format!("{start}{}</c>", value.inline_string_xml())
        }
        TemplateCellValue::Comment { value, .. }
        | TemplateCellValue::Hyperlink { value, .. }
        | TemplateCellValue::Images { value, .. } => {
            render_typed_cell(cell, value, auto_style)
        }
    }
}

// BTreeMap 的键本身包含 Option 以区分默认集合与命名集合；直接借用完整键避免在热路径
// 为查询临时克隆 String。
#[allow(clippy::ref_option)]
fn shift_following_rows_for_fill(
    xml: &mut String,
    cursors: &mut BTreeMap<Option<String>, CollectionFillCursor>,
    key: &Option<String>,
    row_count: usize,
) -> Option<(usize, usize)> {
    let Some(cursor) = cursors.get(key) else {
        return None;
    };
    if cursor.templates.is_empty() {
        return None;
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
        return None;
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
    Some((maximum, shift))
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

#[allow(clippy::too_many_arguments)]
fn positioned_collection_cell(
    template_cell: &str,
    data: &TemplateFillData,
    prefix: Option<&str>,
    shared_strings: &[String],
    auto_style: bool,
    row: usize,
    column: usize,
    style: Option<u32>,
) -> String {
    let mut cell = fill_cell(template_cell, data, prefix, shared_strings, auto_style);
    if let Some(style) = style {
        cell = set_cell_style(&cell, style);
    }
    replace_attribute(
        &cell,
        "r",
        &format!("{}{}", column_name(column + 1), row + 1),
    )
}

fn set_cell_style(cell: &str, style: u32) -> String {
    if attribute_value(cell, "s").is_some() {
        return replace_attribute(cell, "s", &style.to_string());
    }
    let Some(tag_end) = cell.find('>') else {
        return cell.to_owned();
    };
    let mut output = String::with_capacity(cell.len() + 16);
    output.push_str(&cell[..tag_end]);
    let _ = write!(output, " s=\"{style}\"");
    output.push_str(&cell[tag_end..]);
    output
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

fn template_decoration_placements(
    value: &TemplateCellValue,
    row: usize,
    column: usize,
) -> Vec<TemplateDecorationPlacement> {
    let (Ok(row), Ok(column)) = (u32::try_from(row), u16::try_from(column)) else {
        return Vec::new();
    };
    template_value_decorations(value, row, column)
}

/// 返回一个 typed template value 在最终物理坐标上需要落盘的全部装饰。
///
/// Comment、hyperlink 与 images 可以嵌套包装同一显示值；本函数逐层展开，
/// 供 placeholder fill、模板追加和 mutation set-cell 共用同一语义。
#[must_use]
pub fn template_value_decorations(
    value: &TemplateCellValue,
    row: u32,
    column: u16,
) -> Vec<TemplateDecorationPlacement> {
    let mut placements = Vec::new();
    let mut current = value;
    loop {
        match current {
            TemplateCellValue::Comment { value, comment } => {
                placements.push(TemplateDecorationPlacement {
                    row,
                    column,
                    decoration: TemplateDecoration::Comment(comment.clone()),
                });
                current = value;
            }
            TemplateCellValue::Hyperlink { value, hyperlink } => {
                placements.push(TemplateDecorationPlacement {
                    row,
                    column,
                    decoration: TemplateDecoration::Hyperlink(hyperlink.clone()),
                });
                current = value;
            }
            TemplateCellValue::Images { value, images } => {
                placements.extend(images.iter().cloned().map(|image| {
                    TemplateDecorationPlacement {
                        row,
                        column,
                        decoration: TemplateDecoration::Image(image),
                    }
                }));
                current = value;
            }
            _ => break,
        }
    }
    placements
}

fn comment_placements(
    placements: Vec<TemplateDecorationPlacement>,
) -> Vec<TemplateCommentPlacement> {
    placements
        .into_iter()
        .filter_map(|placement| match placement.decoration {
            TemplateDecoration::Comment(comment) => Some(TemplateCommentPlacement {
                row: placement.row,
                column: placement.column,
                comment,
            }),
            TemplateDecoration::Hyperlink(_) => None,
            TemplateDecoration::Image(_) => None,
        })
        .collect()
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

fn replace_scalar_cells_matching_with_decorations(
    entries: &mut [OoxmlZipEntry],
    worksheet: Option<&str>,
    data: &TemplateFillData,
) -> Result<Vec<TemplateDecorationPlacement>> {
    let shared_strings = shared_strings(entries);
    let mut decorations = Vec::new();
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
        let (xml, entry_decorations) =
            replace_scalar_cells_in_xml_with_decorations(&xml, data, &shared_strings);
        entry.bytes = xml.into_bytes();
        decorations.extend(entry_decorations);
    }
    Ok(decorations)
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

#[cfg(test)]
mod tests {
    use super::*;
    use zip::CompressionMethod;

    fn entry(name: &str, bytes: impl Into<Vec<u8>>) -> OoxmlZipEntry {
        OoxmlZipEntry {
            name: name.to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: bytes.into(),
        }
    }

    fn data() -> TemplateFillData {
        TemplateFillData {
            values: BTreeMap::from([
                (
                    "name".to_owned(),
                    TemplateCellValue::Text("<值>".to_owned()),
                ),
                (
                    "number".to_owned(),
                    TemplateCellValue::Number("2".to_owned()),
                ),
            ]),
        }
    }

    #[test]
    fn scalar_and_collection_rendering_stays_inside_xlsx_engine() {
        let data = data();
        assert_eq!(
            replace_template_values("{name}-{missing}", &data.values, None, true, true),
            "&lt;值&gt;-{missing}"
        );
        assert_eq!(
            replace_template_values(r"\{name\}-{name}", &data.values, None, true, true),
            "{name}-&lt;值&gt;"
        );
        assert_eq!(
            fill_cell(
                r#"<c r="A1" s="2" t="inlineStr"><is><t>{.name}</t></is></c>"#,
                &data,
                None,
                &[],
                false,
            ),
            r#"<c r="A1" t="inlineStr"><is><t>&lt;值&gt;</t></is></c>"#
        );
        assert_eq!(
            render_typed_cell(r#"<c r="B1"></c>"#, &TemplateCellValue::Bool(false), true),
            r#"<c r="B1" t="b"><v>0</v></c>"#
        );
        assert!(collection_template_cells("<row", None, &[]).is_empty());
        assert!(
            collection_template_cells(
                r#"<row><c r="bad" t="inlineStr"><is><t>{.name}</t></is></c></row>"#,
                None,
                &[],
            )
            .is_empty()
        );
        assert_eq!(cell_value(r#"<c t="s"><v>9</v></c>"#, &[]), None);
        assert!(!contains_collection_marker("{other.name}", Some("items")));
    }

    #[test]
    fn package_operations_report_missing_and_invalid_worksheet_data() {
        let fill = TemplateCollectionFill {
            name: Some("items".to_owned()),
            rows: vec![data()],
            ..TemplateCollectionFill::default()
        };
        assert!(replace_collection_fills_in_sheet(&mut [], "missing.xml", &[]).is_ok());
        assert!(
            replace_collection_fills_in_sheet(&mut [], "missing.xml", std::slice::from_ref(&fill),)
                .is_err()
        );
        assert!(append_rows_to_xml("<worksheet/>", &[vec![]]).is_err());
        assert!(append_rows_to_sheet(&mut [], "missing.xml", &[vec![]]).is_err());

        let mut invalid = vec![entry("xl/worksheets/sheet1.xml", vec![0xff])];
        assert!(replace_scalar_cells(&mut invalid, &TemplateFillData::default()).is_err());
        invalid[0].bytes = vec![0xff];
        assert!(
            replace_collection_fills_in_sheet(&mut invalid, "xl/worksheets/sheet1.xml", &[fill],)
                .is_err()
        );
    }

    #[test]
    fn repeated_collection_fill_shifts_cached_templates_and_dimension() {
        let worksheet = concat!(
            r#"<worksheet><dimension ref="A1:A5"/><sheetData>"#,
            r#"<row r="1"><c r="A1" t="inlineStr"><is><t>{a.name}</t></is></c></row>"#,
            r#"<row r="3"><c r="A3" t="inlineStr"><is><t>{b.name}</t></is></c></row>"#,
            r#"<row r="5"><c r="A5" t="inlineStr"><is><t>Footer</t></is></c></row>"#,
            "</sheetData></worksheet>",
        );
        let mut entries = vec![entry("xl/worksheets/sheet1.xml", worksheet.as_bytes())];
        let fills = [
            TemplateCollectionFill {
                name: Some("b".to_owned()),
                rows: vec![TemplateFillData {
                    values: BTreeMap::from([(
                        "name".to_owned(),
                        TemplateCellValue::Text("B1".to_owned()),
                    )]),
                }],
                order: 0,
                ..TemplateCollectionFill::default()
            },
            TemplateCollectionFill {
                name: Some("a".to_owned()),
                rows: vec![
                    TemplateFillData {
                        values: BTreeMap::from([(
                            "name".to_owned(),
                            TemplateCellValue::Text("A1".to_owned()),
                        )]),
                    },
                    TemplateFillData {
                        values: BTreeMap::from([(
                            "name".to_owned(),
                            TemplateCellValue::Text("A2".to_owned()),
                        )]),
                    },
                ],
                force_new_row: true,
                order: 1,
                ..TemplateCollectionFill::default()
            },
            TemplateCollectionFill {
                name: Some("b".to_owned()),
                rows: vec![TemplateFillData {
                    values: BTreeMap::from([(
                        "name".to_owned(),
                        TemplateCellValue::Text("B2".to_owned()),
                    )]),
                }],
                order: 2,
                ..TemplateCollectionFill::default()
            },
        ];

        replace_collection_fills_in_sheet(&mut entries, "xl/worksheets/sheet1.xml", &fills)
            .expect("fill collections");
        let xml = std::str::from_utf8(&entries[0].bytes).expect("worksheet UTF-8");
        assert!(xml.contains("A2"));
        assert!(xml.contains("B2"));
        assert!(xml.contains(r#"ref="A1:A6""#));
    }
}
