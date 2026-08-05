//! OOXML 占位符填充引擎（集合/标量展开、行追加与引用平移）。
//!
//! 对应 Java：内部辅助类型（OOXML 占位符展开引擎）

use std::collections::BTreeMap;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use crate::core::{CellValue, ExcelError, Result};

use crate::template::sheet_fill_state::PendingCollectionFill;
use crate::template::template_entry::TemplateEntry;
use crate::{FillDirection, FillWrapper, TemplateData};

#[cfg(test)]
use crate::FillConfig;

#[derive(Debug, Clone)]
pub(crate) struct CollectionTemplateCell {
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

// 集合填充按“占位符解析 → 模板展开 → 游标缓存 → 行平移”线性推进，
// 拆分会把跨步骤共享的 shared_strings/rows 状态拆散，故豁免 too_many_lines。
#[allow(clippy::too_many_lines)]
pub(crate) fn replace_collection_fills_in_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    fills: &[PendingCollectionFill],
) -> Result<()> {
    if fills.is_empty() {
        return Ok(());
    }
    let shared_strings = entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case("xl/sharedStrings.xml"))
        .and_then(|entry| std::str::from_utf8(&entry.bytes).ok())
        .map(shared_string_values)
        .unwrap_or_default();
    let Some(entry) = entries
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(worksheet))
    else {
        return Err(ExcelError::Format(format!(
            "worksheet part {worksheet:?} is missing"
        )));
    };
    let mut xml = std::str::from_utf8(&entry.bytes)
        .map_err(|error| ExcelError::Format(error.to_string()))?
        .to_owned();
    let mut cursors: BTreeMap<Option<String>, CollectionFillCursor> = BTreeMap::new();

    for fill in fills {
        let key = fill.wrapper.name.clone();
        if !cursors.contains_key(&key) {
            let templates = collection_template_cells(&xml, &fill.wrapper, &shared_strings);
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
        if fill.config.get_direction() == FillDirection::Vertical && fill.config.get_force_new_row()
        {
            let cursor = cursors
                .get(&key)
                .expect("collection cursor was initialized");
            if !cursor.templates.is_empty() {
                let max_row = cursor
                    .templates
                    .iter()
                    .zip(&cursor.last_indices)
                    .map(|(template, last)| last.unwrap_or(template.row))
                    .max()
                    .unwrap_or(0);
                let shift = fill
                    .wrapper
                    .rows()
                    .len()
                    .saturating_sub(usize::from(!cursor.initialized));
                if shift > 0 && max_row < last_worksheet_row(&xml).unwrap_or(max_row) {
                    xml = shift_worksheet_rows_after(&xml, max_row, shift);
                    for cached in cursors.values_mut() {
                        for template in &mut cached.templates {
                            if template.row > max_row {
                                template.row = template.row.saturating_add(shift);
                                template.row_tag = replace_attribute(
                                    &template.row_tag,
                                    "r",
                                    &(template.row + 1).to_string(),
                                );
                            }
                        }
                    }
                }
            }
        }

        let cursor = cursors
            .get_mut(&key)
            .expect("collection cursor was initialized");
        for data in fill.wrapper.rows() {
            for index in 0..cursor.templates.len() {
                let template = cursor.templates[index].clone();
                let (target_row, target_column, last_index) = match fill.config.get_direction() {
                    FillDirection::Vertical => {
                        let row = cursor.last_indices[index]
                            .map_or(template.row, |last| last.saturating_add(1));
                        (row, template.column, row)
                    }
                    FillDirection::Horizontal => {
                        let column = cursor.last_indices[index]
                            .map_or(template.column, |last| last.saturating_add(1));
                        (template.row, column, column)
                    }
                };
                validate_collection_target(target_row, target_column)?;
                let cell = positioned_collection_cell(
                    &template.cell,
                    data,
                    fill.wrapper.name(),
                    &shared_strings,
                    fill.config.get_auto_style(),
                    target_row,
                    target_column,
                );
                let row_tag =
                    replace_attribute(&template.row_tag, "r", &(target_row + 1).to_string());
                let row = format!("{row_tag}{cell}</row>");
                xml = upsert_collection_row(&xml, &row, target_row + 1);
                cursor.last_indices[index] = Some(last_index);
            }
            cursor.initialized = true;
        }
    }
    entry.bytes = update_worksheet_dimension(&xml).into_bytes();
    Ok(())
}

pub(crate) fn collection_template_cells(
    xml: &str,
    wrapper: &FillWrapper,
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
        let tag_end = row_xml
            .find('>')
            .expect("a row with a closing tag contains a tag terminator");
        for (_, _, cell) in collection_cells(row_xml, wrapper, shared_strings) {
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

pub(crate) fn row_tag_with_reference(row_tag: &str, row: usize) -> String {
    if attribute_value(row_tag, "r").is_some() {
        row_tag.to_owned()
    } else {
        replace_attribute(row_tag, "r", &row.to_string())
    }
}

fn positioned_collection_cell(
    template_cell: &str,
    data: &TemplateData,
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

pub(crate) fn validate_collection_target(row: usize, column: usize) -> Result<()> {
    if row >= 1_048_576 {
        return Err(ExcelError::Format(
            "collection fill row exceeds XLSX limit".to_owned(),
        ));
    }
    if column >= 16_384 {
        return Err(ExcelError::Format(
            "collection fill column exceeds XLSX limit".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn last_worksheet_row(xml: &str) -> Option<usize> {
    let mut last = None;
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find("</row>") else {
            break;
        };
        let end = start + relative_end + 6;
        if let Some(row) = row_index(&xml[start..end]) {
            last = Some(last.map_or(row, |current: usize| current.max(row)));
        }
        offset = end;
    }
    last.map(|row| row - 1)
}

pub(crate) fn shift_worksheet_rows_after(xml: &str, row: usize, delta: usize) -> String {
    let threshold = row + 2;
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find("</row>") else {
            break;
        };
        let end = start + relative_end + 6;
        if row_index(&xml[start..end]).is_some_and(|candidate| candidate >= threshold) {
            let shifted = format!("{}{}", &xml[..start], shift_rows(&xml[start..], delta));
            return shift_worksheet_metadata(&shifted, threshold, delta);
        }
        offset = end;
    }
    xml.to_owned()
}

#[cfg(test)]
pub(crate) fn replace_collection_placeholders(
    entries: &mut [TemplateEntry],
    wrapper: &FillWrapper,
    config: FillConfig,
) {
    replace_collection_placeholders_matching(entries, None, wrapper, config);
}

#[cfg(test)]
pub(crate) fn replace_collection_placeholders_matching(
    entries: &mut [TemplateEntry],
    worksheet: Option<&str>,
    wrapper: &FillWrapper,
    config: FillConfig,
) {
    if wrapper.rows().is_empty() {
        return;
    }
    let shared_strings = entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case("xl/sharedStrings.xml"))
        .and_then(|entry| std::str::from_utf8(&entry.bytes).ok())
        .map(shared_string_values)
        .unwrap_or_default();
    for entry in entries.iter_mut().filter(|entry| {
        worksheet.map_or_else(
            || entry.name.starts_with("xl/worksheets/"),
            |worksheet| entry.name.eq_ignore_ascii_case(worksheet),
        ) && Path::new(&entry.name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
    }) {
        let Ok(xml) = std::str::from_utf8(&entry.bytes) else {
            continue;
        };
        let expanded = match config.get_direction() {
            FillDirection::Vertical => expand_vertical_rows(xml, wrapper, config, &shared_strings),
            FillDirection::Horizontal => expand_horizontal_cells(xml, wrapper, &shared_strings),
        };
        if let Some(expanded) = expanded {
            entry.bytes = update_worksheet_dimension(&expanded).into_bytes();
            break;
        }
    }
}

pub(crate) fn shared_string_values(xml: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut remaining = xml;
    while let Some(start) = remaining.find("<si") {
        let item = &remaining[start..];
        let Some(open_end) = item.find('>') else {
            break;
        };
        let Some(close) = item.find("</si>") else {
            break;
        };
        values.push(text_node_values(&item[open_end + 1..close]));
        remaining = &item[close + 5..];
    }
    values
}

pub(crate) fn text_node_values(xml: &str) -> String {
    let mut value = String::new();
    let mut remaining = xml;
    while let Some(start) = remaining.find("<t") {
        let text = &remaining[start..];
        let Some(open_end) = text.find('>') else {
            break;
        };
        let Some(close) = text.find("</t>") else {
            break;
        };
        value.push_str(&text[open_end + 1..close]);
        remaining = &text[close + 4..];
    }
    value
}

#[cfg(test)]
pub(crate) fn expand_vertical_rows(
    xml: &str,
    wrapper: &FillWrapper,
    config: FillConfig,
    shared_strings: &[String],
) -> Option<String> {
    let (start, end, row, _, _, _) = find_collection_row(xml, wrapper, shared_strings)?;
    let first = fill_row_cells(
        row,
        wrapper.rows().first()?,
        wrapper.name(),
        shared_strings,
        config.get_auto_style(),
    );
    if config.get_force_new_row() {
        let template_row = row_index(row)?;
        let mut rows = first;
        for (offset, data) in wrapper.rows().iter().enumerate().skip(1) {
            rows.push_str(&collection_only_row(
                row,
                data,
                wrapper,
                shared_strings,
                config.get_auto_style(),
                offset,
            ));
        }
        let delta = wrapper.rows().len().saturating_sub(1);
        let suffix = shift_rows(&xml[end..], delta);
        let expanded = format!("{}{}{}", &xml[..start], rows, suffix);
        return Some(shift_worksheet_metadata(&expanded, template_row + 1, delta));
    }

    let template_row = row_index(row)?;
    let mut suffix = xml[end..].to_owned();
    for (offset, data) in wrapper.rows().iter().enumerate().skip(1) {
        let row = collection_only_row(
            row,
            data,
            wrapper,
            shared_strings,
            config.get_auto_style(),
            offset,
        );
        suffix = upsert_collection_row(&suffix, &row, template_row + offset);
    }
    Some(format!("{}{}{}", &xml[..start], first, suffix))
}

#[cfg(test)]
pub(crate) fn collection_only_row(
    template_row: &str,
    data: &TemplateData,
    wrapper: &FillWrapper,
    shared_strings: &[String],
    auto_style: bool,
    row_offset: usize,
) -> String {
    let Some(tag_end) = template_row.find('>') else {
        return template_row.to_owned();
    };
    let mut row = shift_row(&template_row[..=tag_end], row_offset, 0);
    for (_, _, cell) in collection_cells(template_row, wrapper, shared_strings) {
        let filled = fill_cell(cell, data, wrapper.name(), shared_strings, auto_style);
        row.push_str(&shift_row(&filled, row_offset, 0));
    }
    row.push_str("</row>");
    row
}

pub(crate) fn collection_cells<'a>(
    row: &'a str,
    wrapper: &FillWrapper,
    shared_strings: &[String],
) -> Vec<(usize, usize, &'a str)> {
    all_cells(row)
        .into_iter()
        .filter(|(_, _, cell)| {
            cell_value(cell, shared_strings)
                .is_some_and(|value| contains_collection_marker(&value, wrapper))
        })
        .collect()
}

pub(crate) fn upsert_collection_row(xml: &str, collection_row: &str, target_row: usize) -> String {
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find("</row>") else {
            break;
        };
        let end = start + relative_end + 6;
        let existing = &xml[start..end];
        match row_index(existing) {
            Some(row) if row == target_row => {
                let merged = merge_collection_cells(existing, collection_row);
                return format!("{}{}{}", &xml[..start], merged, &xml[end..]);
            }
            Some(row) if row > target_row => {
                return format!("{}{}{}", &xml[..start], collection_row, &xml[start..]);
            }
            _ => offset = end,
        }
    }
    if let Some(end) = xml.find("</sheetData>") {
        return format!("{}{}{}", &xml[..end], collection_row, &xml[end..]);
    }
    format!("{xml}{collection_row}")
}

pub(crate) fn merge_collection_cells(existing_row: &str, collection_row: &str) -> String {
    let mut merged = existing_row.to_owned();
    for (_, _, cell) in all_cells(collection_row) {
        let Some(reference) = attribute_value(cell, "r") else {
            continue;
        };
        if let Some((start, end, _)) = all_cells(&merged)
            .into_iter()
            .find(|(_, _, existing)| attribute_value(existing, "r") == Some(reference))
        {
            merged.replace_range(start..end, cell);
        } else if let Some(end) = merged.rfind("</row>") {
            merged.insert_str(end, cell);
        }
    }
    merged
}

pub(crate) fn all_cells(row: &str) -> Vec<(usize, usize, &str)> {
    let mut cells = Vec::new();
    let mut offset = 0;
    while let Some((start, end)) = find_next_cell(row, offset) {
        cells.push((start, end, &row[start..end]));
        offset = end;
    }
    cells
}

/// Finds the next OOXML cell element (`<c ...>` / `<c .../>`).
///
/// Must not match similarly-prefixed tags such as `<cols>` / `<col>` — those
/// false positives previously rewrote worksheet XML during scalar fill and
/// left `complex.xlsx` unreadable by `quick_xml`.
fn find_next_cell(xml: &str, from: usize) -> Option<(usize, usize)> {
    let bytes = xml.as_bytes();
    let mut search = from;
    while search < xml.len() {
        let relative = xml[search..].find("<c")?;
        let start = search + relative;
        let after_c = start + 2;
        let next = *bytes.get(after_c)?;
        // Local name must be exactly `c` (followed by space, `/`, or `>`).
        if !matches!(next, b' ' | b'\t' | b'\n' | b'\r' | b'/' | b'>') {
            search = after_c;
            continue;
        }
        let relative_gt = xml[after_c..].find('>')?;
        let gt = after_c + relative_gt;
        // Self-closing `<c .../>` — do not scan forward for `</c>`.
        if gt > start && bytes[gt - 1] == b'/' {
            return Some((start, gt + 1));
        }
        let relative_end = xml[gt..].find("</c>")?;
        let end = gt + relative_end + 4;
        return Some((start, end));
    }
    None
}

pub(crate) fn row_index(row: &str) -> Option<usize> {
    attribute_value(row, "r")?.parse().ok()
}

pub(crate) fn update_worksheet_dimension(xml: &str) -> String {
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    for (_, _, cell) in all_cells(xml) {
        let Some(reference) = attribute_value(cell, "r") else {
            continue;
        };
        let Some((column, row)) = parse_cell_reference(reference) else {
            continue;
        };
        bounds = Some(bounds.map_or((column, row, column, row), |current| {
            (
                current.0.min(column),
                current.1.min(row),
                current.2.max(column),
                current.3.max(row),
            )
        }));
    }
    let Some((first_column, first_row, last_column, last_row)) = bounds else {
        return xml.to_owned();
    };
    let reference = format!(
        "{}{}:{}{}",
        column_name(first_column),
        first_row,
        column_name(last_column),
        last_row
    );
    replace_tag_attribute(xml, "dimension", "ref", &reference)
}

pub(crate) fn shift_worksheet_metadata(xml: &str, threshold_row: usize, delta: usize) -> String {
    if delta == 0 {
        return xml.to_owned();
    }
    let mut shifted = xml.to_owned();
    for (tag, attribute) in [
        ("mergeCell", "ref"),
        ("hyperlink", "ref"),
        ("autoFilter", "ref"),
        ("dataValidation", "sqref"),
        ("conditionalFormatting", "sqref"),
    ] {
        shifted = shift_tag_references(&shifted, tag, attribute, threshold_row, delta);
    }
    shift_formula_elements(&shifted, threshold_row, delta)
}

pub(crate) fn shift_tag_references(
    xml: &str,
    tag: &str,
    attribute: &str,
    threshold_row: usize,
    delta: usize,
) -> String {
    let mut output = String::new();
    let mut offset = 0;
    let marker = format!("<{tag}");
    while let Some(relative_start) = xml[offset..].find(&marker) {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find('>') else {
            break;
        };
        let end = start + relative_end + 1;
        output.push_str(&xml[offset..start]);
        let element = &xml[start..end];
        let shifted = attribute_value(element, attribute).map_or_else(
            || element.to_owned(),
            |value| {
                replace_attribute(
                    element,
                    attribute,
                    &shift_reference_list(value, threshold_row, delta),
                )
            },
        );
        output.push_str(&shifted);
        offset = end;
    }
    output.push_str(&xml[offset..]);
    output
}

pub(crate) fn shift_reference_list(value: &str, threshold_row: usize, delta: usize) -> String {
    value
        .split_whitespace()
        .map(|range| {
            range
                .split(':')
                .map(|reference| shift_a1_reference(reference, threshold_row, delta))
                .collect::<Vec<_>>()
                .join(":")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn shift_formula_elements(xml: &str, threshold_row: usize, delta: usize) -> String {
    let mut output = String::new();
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<f") {
        let start = offset + relative_start;
        let Some(open_end) = xml[start..].find('>') else {
            break;
        };
        let content_start = start + open_end + 1;
        let Some(relative_end) = xml[content_start..].find("</f>") else {
            break;
        };
        let content_end = content_start + relative_end;
        output.push_str(&xml[offset..content_start]);
        output.push_str(&shift_formula_references(
            &xml[content_start..content_end],
            threshold_row,
            delta,
        ));
        offset = content_end;
    }
    output.push_str(&xml[offset..]);
    output
}

pub(crate) fn shift_formula_references(
    formula: &str,
    threshold_row: usize,
    delta: usize,
) -> String {
    let bytes = formula.as_bytes();
    let mut output = String::new();
    let mut offset = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'$' && !bytes[index].is_ascii_alphabetic() {
            index += 1;
            continue;
        }
        let start = index;
        if bytes[index] == b'$' {
            index += 1;
        }
        let column_start = index;
        while index < bytes.len() && bytes[index].is_ascii_alphabetic() {
            index += 1;
        }
        let column_end = index;
        if index < bytes.len() && bytes[index] == b'$' {
            index += 1;
        }
        let row_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        let valid = column_end > column_start
            && row_start < index
            && column_end - column_start <= 3
            && (start == 0 || !is_formula_identifier(bytes[start - 1]))
            && (index == bytes.len()
                || (!is_formula_identifier(bytes[index])
                    && bytes[index] != b'!'
                    && bytes[index] != b'('));
        if valid {
            output.push_str(&formula[offset..start]);
            output.push_str(&shift_a1_reference(
                &formula[start..index],
                threshold_row,
                delta,
            ));
            offset = index;
        }
    }
    output.push_str(&formula[offset..]);
    output
}

const fn is_formula_identifier(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub(crate) fn shift_a1_reference(reference: &str, threshold_row: usize, delta: usize) -> String {
    let Some((column, row)) = parse_cell_reference(reference) else {
        return reference.to_owned();
    };
    if row < threshold_row {
        return reference.to_owned();
    }
    let absolute_column = reference.starts_with('$');
    let row_marker = reference
        .bytes()
        .position(|byte| byte.is_ascii_digit())
        .is_some_and(|index| index > 0 && reference.as_bytes()[index - 1] == b'$');
    format!(
        "{}{}{}{}",
        if absolute_column { "$" } else { "" },
        column_name(column),
        if row_marker { "$" } else { "" },
        row + delta
    )
}

pub(crate) fn parse_cell_reference(reference: &str) -> Option<(usize, usize)> {
    let normalized = reference.replace('$', "");
    let split = normalized.bytes().position(|byte| byte.is_ascii_digit())?;
    if split == 0
        || !normalized[..split]
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic())
    {
        return None;
    }
    let column = normalized[..split]
        .bytes()
        .try_fold(0_usize, |value, byte| {
            value
                .checked_mul(26)?
                .checked_add(usize::from(byte.to_ascii_uppercase() - b'A' + 1))
        })?;
    if column > 16_384 {
        return None;
    }
    let row = normalized[split..].parse::<usize>().ok()?;
    (row > 0).then_some((column, row))
}

pub(crate) fn replace_tag_attribute(xml: &str, tag: &str, attribute: &str, value: &str) -> String {
    let marker = format!("<{tag}");
    let Some(start) = xml.find(&marker) else {
        return xml.to_owned();
    };
    let Some(relative_end) = xml[start..].find('>') else {
        return xml.to_owned();
    };
    let end = start + relative_end + 1;
    let replaced = replace_attribute(&xml[start..end], attribute, value);
    format!("{}{}{}", &xml[..start], replaced, &xml[end..])
}

#[cfg(test)]
pub(crate) fn expand_horizontal_cells(
    xml: &str,
    wrapper: &FillWrapper,
    shared_strings: &[String],
) -> Option<String> {
    let mut output = String::with_capacity(xml.len());
    let mut offset = 0;
    let mut changed = false;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find("</row>") else {
            break;
        };
        let end = start + relative_end + 6;
        output.push_str(&xml[offset..start]);
        let row = &xml[start..end];
        let cells = collection_cells(row, wrapper, shared_strings);
        if cells.is_empty() {
            output.push_str(row);
        } else {
            changed = true;
            let mut cell_offset = 0;
            for (cell_start, cell_end, cell) in cells {
                output.push_str(&row[cell_offset..cell_start]);
                for (column_offset, data) in wrapper.rows().iter().enumerate() {
                    let filled = fill_cell(cell, data, wrapper.name(), shared_strings, true);
                    output.push_str(&shift_row(&filled, 0, column_offset));
                }
                cell_offset = cell_end;
            }
            output.push_str(&row[cell_offset..]);
        }
        offset = end;
    }
    output.push_str(&xml[offset..]);
    changed.then_some(output)
}

#[cfg(test)]
pub(crate) fn find_collection_row<'a>(
    xml: &'a str,
    wrapper: &FillWrapper,
    shared_strings: &[String],
) -> Option<(usize, usize, &'a str, usize, usize, &'a str)> {
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let end = start + xml[start..].find("</row>")? + 6;
        let row = &xml[start..end];
        if let Some((cell_start, cell_end, cell)) =
            find_collection_cell(row, wrapper, shared_strings)
        {
            return Some((start, end, row, cell_start, cell_end, cell));
        }
        offset = end;
    }
    None
}

#[cfg(test)]
pub(crate) fn find_collection_cell<'a>(
    row: &'a str,
    wrapper: &FillWrapper,
    shared_strings: &[String],
) -> Option<(usize, usize, &'a str)> {
    let mut offset = 0;
    while let Some((start, end)) = find_next_cell(row, offset) {
        let cell = &row[start..end];
        if cell_value(cell, shared_strings)
            .is_some_and(|value| contains_collection_marker(&value, wrapper))
        {
            return Some((start, end, cell));
        }
        offset = end;
    }
    None
}

#[cfg(test)]
pub(crate) fn fill_row_cells(
    row: &str,
    data: &TemplateData,
    prefix: Option<&str>,
    shared_strings: &[String],
    auto_style: bool,
) -> String {
    let mut output = String::new();
    let mut offset = 0;
    while let Some((start, end)) = find_next_cell(row, offset) {
        output.push_str(&row[offset..start]);
        output.push_str(&fill_cell(
            &row[start..end],
            data,
            prefix,
            shared_strings,
            auto_style,
        ));
        offset = end;
    }
    output.push_str(&row[offset..]);
    output
}

pub(crate) fn fill_cell(
    cell: &str,
    data: &TemplateData,
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
    let filled = replace_collection_values(&value, data, prefix);
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

pub(crate) fn exact_collection_value<'a>(
    placeholder: &str,
    data: &'a TemplateData,
    prefix: Option<&str>,
) -> Option<&'a CellValue> {
    let variable = placeholder.strip_prefix('{')?.strip_suffix('}')?;
    let key = match prefix {
        Some(prefix) => variable.strip_prefix(prefix)?.strip_prefix('.')?,
        None => variable.strip_prefix('.')?,
    };
    data.values().get(key)
}

fn exact_scalar_value<'a>(placeholder: &str, data: &'a TemplateData) -> Option<&'a CellValue> {
    let key = placeholder.strip_prefix('{')?.strip_suffix('}')?;
    (!key.starts_with('.') && !key.ends_with('.'))
        .then(|| data.values().get(key))
        .flatten()
}

pub(crate) fn render_typed_cell(cell: &str, value: &CellValue, auto_style: bool) -> String {
    let Some(tag_end) = cell.find('>') else {
        return cell.to_owned();
    };
    let mut start = cell[..=tag_end].to_owned();
    if !auto_style {
        start = remove_attribute(&start, "s");
    }
    start = remove_attribute(&start, "t");
    match value {
        CellValue::Empty | CellValue::Image(_) => format!("{start}</c>"),
        CellValue::String(value) | CellValue::Hyperlink { text: value, .. } => {
            insert_cell_type(&mut start, "inlineStr");
            format!("{start}<is><t>{}</t></is></c>", escape_xml(value))
        }
        CellValue::Bool(value) => {
            insert_cell_type(&mut start, "b");
            format!("{start}<v>{}</v></c>", u8::from(*value))
        }
        CellValue::Int(value) => format!("{start}<v>{value}</v></c>"),
        CellValue::Float(value) => format!("{start}<v>{value}</v></c>"),
        CellValue::Decimal(value) => format!("{start}<v>{value}</v></c>"),
        CellValue::Date(value) => {
            insert_cell_type(&mut start, "d");
            format!("{start}<v>{}</v></c>", value.format("%Y-%m-%d"))
        }
        CellValue::DateTime(value) => {
            insert_cell_type(&mut start, "d");
            format!("{start}<v>{}</v></c>", value.format("%Y-%m-%dT%H:%M:%S"))
        }
        CellValue::Error(value) => {
            insert_cell_type(&mut start, "e");
            format!("{start}<v>{}</v></c>", escape_xml(value))
        }
        CellValue::Formula(value) => {
            format!("{start}<f>{}</f><v></v></c>", escape_xml(value))
        }
        CellValue::RichText(value) => {
            insert_cell_type(&mut start, "inlineStr");
            format!(
                "{start}<is><t>{}</t></is></c>",
                escape_xml(value.text_string())
            )
        }
        CellValue::Comment { value, .. } | CellValue::Images { value, .. } => {
            render_typed_cell(cell, value, auto_style)
        }
    }
}

fn insert_cell_type(start: &mut String, cell_type: &str) {
    start.insert_str(start.len() - 1, &format!(" t=\"{cell_type}\""));
}

pub(crate) fn cell_value(cell: &str, shared_strings: &[String]) -> Option<String> {
    if attribute_value(cell, "t") == Some("s") {
        let index = element_value(cell, "v")?.parse::<usize>().ok()?;
        return shared_strings.get(index).cloned();
    }
    let value = text_node_values(cell);
    (!value.is_empty()).then_some(value)
}

pub(crate) fn contains_collection_marker(value: &str, wrapper: &FillWrapper) -> bool {
    let prefix = wrapper
        .name()
        .map_or(".".to_owned(), |name| format!("{name}."));
    contains_unescaped(value, &format!("{{{prefix}"))
}

fn replace_collection_values(value: &str, data: &TemplateData, prefix: Option<&str>) -> String {
    replace_template_values(value, data.values(), prefix, false, false)
}

#[cfg(test)]
pub(crate) fn replace_scalar_cells(
    entries: &mut [TemplateEntry],
    data: &TemplateData,
) -> Result<()> {
    replace_scalar_cells_matching(entries, None, data)
}

pub(crate) fn replace_scalar_cells_in_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    data: &TemplateData,
) -> Result<()> {
    replace_scalar_cells_matching(entries, Some(worksheet), data)
}

fn replace_scalar_cells_matching(
    entries: &mut [TemplateEntry],
    worksheet: Option<&str>,
    data: &TemplateData,
) -> Result<()> {
    let shared_strings = entries
        .iter()
        .find(|entry| entry.name == "xl/sharedStrings.xml")
        .and_then(|entry| std::str::from_utf8(&entry.bytes).ok())
        .map_or_else(Vec::new, shared_string_values);
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
            .map_err(|error| ExcelError::Format(error.to_string()))?;
        entry.bytes = replace_scalar_cells_in_xml(&xml, data, &shared_strings).into_bytes();
    }
    Ok(())
}

pub(crate) fn replace_scalar_cells_in_xml(
    xml: &str,
    data: &TemplateData,
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
                let filled =
                    replace_template_values(&placeholder, data.values(), None, true, false);
                if filled == placeholder {
                    cell.to_owned()
                } else {
                    render_typed_cell(cell, &CellValue::String(filled), true)
                }
            },
        );
        output.push_str(&replacement);
        offset = end;
    }
    output.push_str(&xml[offset..]);
    output
}

#[cfg(test)]
pub(crate) fn append_rows_to_first_sheet(
    entries: &mut [TemplateEntry],
    rows: &[Vec<CellValue>],
) -> Result<()> {
    append_rows_to_sheet(entries, "xl/worksheets/sheet1.xml", rows)
}

pub(crate) fn append_rows_to_sheet(
    entries: &mut [TemplateEntry],
    worksheet: &str,
    rows: &[Vec<CellValue>],
) -> Result<()> {
    if rows.is_empty() {
        return Ok(());
    }
    let Some(entry) = entries
        .iter_mut()
        .find(|entry| entry.name.eq_ignore_ascii_case(worksheet))
    else {
        return Err(ExcelError::Format(format!(
            "template does not contain {worksheet}"
        )));
    };
    let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
        .map_err(|error| ExcelError::Format(error.to_string()))?;
    entry.bytes = append_rows_to_xml(&xml, rows)?.into_bytes();
    Ok(())
}

pub(crate) fn append_rows_to_xml(xml: &str, rows: &[Vec<CellValue>]) -> Result<String> {
    let Some(sheet_data_end) = xml.find("</sheetData>") else {
        return Err(ExcelError::Format(
            "worksheet does not contain sheetData".to_owned(),
        ));
    };
    let next_row = worksheet_max_row(&xml[..sheet_data_end]).saturating_add(1);
    let mut appended = String::new();
    for (row_offset, values) in rows.iter().enumerate() {
        let row_index = next_row + row_offset;
        write!(appended, "<row r=\"{row_index}\">").expect("writing to String cannot fail");
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

pub(crate) fn worksheet_max_row(xml: &str) -> usize {
    let mut maximum = 0;
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find('>') else {
            break;
        };
        let end = start + relative_end + 1;
        if let Some(row) = row_index(&xml[start..end]) {
            maximum = maximum.max(row);
        }
        offset = end;
    }
    maximum
}

pub(crate) fn element_value<'a>(xml: &'a str, element: &str) -> Option<&'a str> {
    let start_marker = format!("<{element}>");
    let end_marker = format!("</{element}>");
    let start = xml.find(&start_marker)? + start_marker.len();
    let end = start + xml[start..].find(&end_marker)?;
    Some(&xml[start..end])
}

pub(crate) fn attribute_value<'a>(xml: &'a str, attribute: &str) -> Option<&'a str> {
    let marker = format!(" {attribute}=\"");
    let start = xml.find(&marker)? + marker.len();
    let end = start + xml[start..].find('"')?;
    Some(&xml[start..end])
}

pub(crate) fn replace_attribute(xml: &str, attribute: &str, value: &str) -> String {
    let Some(current) = attribute_value(xml, attribute) else {
        return xml.to_owned();
    };
    xml.replacen(
        &format!(" {attribute}=\"{current}\""),
        &format!(" {attribute}=\"{value}\""),
        1,
    )
}

pub(crate) fn remove_attribute(xml: &str, attribute: &str) -> String {
    let Some(current) = attribute_value(xml, attribute) else {
        return xml.to_owned();
    };
    xml.replacen(&format!(" {attribute}=\"{current}\""), "", 1)
}

pub(crate) fn shift_rows(xml: &str, delta: usize) -> String {
    if delta == 0 {
        return xml.to_owned();
    }
    let mut output = String::new();
    let mut offset = 0;
    while let Some(relative_start) = xml[offset..].find("<row") {
        let start = offset + relative_start;
        let Some(relative_end) = xml[start..].find("</row>") else {
            break;
        };
        let end = start + relative_end + 6;
        output.push_str(&xml[offset..start]);
        output.push_str(&shift_row(&xml[start..end], delta, 0));
        offset = end;
    }
    output.push_str(&xml[offset..]);
    output
}

pub(crate) fn shift_row(xml: &str, row_delta: usize, column_delta: usize) -> String {
    let mut shifted = xml.to_owned();
    let references = cell_references(xml);
    for reference in references.into_iter().rev() {
        let replacement = shift_cell_reference(reference.2, row_delta, column_delta);
        shifted.replace_range(reference.0..reference.1, &replacement);
    }
    if xml.starts_with("<row")
        && let Some(row) = attribute_value(xml, "r").and_then(|value| value.parse::<usize>().ok())
    {
        shifted = replace_attribute(&shifted, "r", &(row + row_delta).to_string());
    }
    shifted
}

pub(crate) fn cell_references(xml: &str) -> Vec<(usize, usize, &str)> {
    let mut references = Vec::new();
    let mut offset = 0;
    while let Some(relative) = xml[offset..].find(" r=\"") {
        let start = offset + relative + 4;
        let Some(length) = xml[start..].find('"') else {
            break;
        };
        let end = start + length;
        let value = &xml[start..end];
        if value.bytes().any(|byte| byte.is_ascii_alphabetic())
            && value.bytes().any(|byte| byte.is_ascii_digit())
        {
            references.push((start, end, value));
        }
        offset = end + 1;
    }
    references
}

pub(crate) fn shift_cell_reference(
    reference: &str,
    row_delta: usize,
    column_delta: usize,
) -> String {
    let split = reference
        .bytes()
        .position(|byte| byte.is_ascii_digit())
        .unwrap_or(reference.len());
    if split == 0
        || split == reference.len()
        || !reference[..split]
            .bytes()
            .all(|byte| byte.is_ascii_alphabetic())
    {
        return reference.to_owned();
    }
    let column = reference[..split].bytes().fold(0_usize, |value, byte| {
        value * 26 + usize::from(byte.to_ascii_uppercase() - b'A' + 1)
    });
    let Ok(row) = reference[split..].parse::<usize>() else {
        return reference.to_owned();
    };
    let row = row + row_delta;
    format!("{}{}", column_name(column + column_delta), row)
}

pub(crate) fn column_name(mut column: usize) -> String {
    let mut name = String::new();
    while column > 0 {
        column -= 1;
        name.insert(0, char::from(b'A' + u8::try_from(column % 26).unwrap_or(0)));
        column /= 26;
    }
    name
}

#[cfg(test)]
pub(crate) fn replace_placeholders(xml: &str, values: &BTreeMap<String, CellValue>) -> String {
    replace_template_values(xml, values, None, true, true)
}

fn replace_template_values(
    input: &str,
    values: &BTreeMap<String, CellValue>,
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

pub(crate) fn contains_unescaped(value: &str, marker: &str) -> bool {
    let mut offset = 0;
    while let Some(relative) = value[offset..].find(marker) {
        let index = offset + relative;
        let backslashes = value[..index]
            .bytes()
            .rev()
            .take_while(|byte| *byte == b'\\')
            .count();
        if backslashes % 2 == 0 {
            return true;
        }
        offset = index + marker.len();
    }
    false
}

pub(crate) fn escape_xml(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests_extra {
    use super::*;
    use zip::CompressionMethod;

    /// 对应 Java：`last_worksheet_row` 在无 `r` 属性的行上走 if-let 的 else 边。
    #[test]
    fn last_worksheet_row_skips_rows_without_reference() {
        assert_eq!(last_worksheet_row("<row></row>"), None);
        assert_eq!(
            last_worksheet_row(r#"<row r="2"></row><row></row>"#),
            Some(1)
        );
        assert_eq!(
            last_worksheet_row(r#"<row r="3"></row><row r="7"></row>"#),
            Some(6)
        );
    }

    /// 对应 Java：`replace_collection_placeholders` 指定 worksheet 过滤 +
    /// 水平方向展开（覆盖 `expand_horizontal_cells` 全部主体）。
    #[test]
    fn replace_collection_placeholders_matching_filters_sheet_and_expands_horizontal() {
        let wrapper = FillWrapper::new([
            TemplateData::new().with("name", "A"),
            TemplateData::new().with("name", "B"),
        ]);
        let worksheet = r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>{.name}</t></is></c></row><row r="2"><c r="A2" t="inlineStr"><is><t>static</t></is></c></row></sheetData></worksheet>"#;
        let mut entries = vec![
            TemplateEntry {
                name: "xl/worksheets/sheet1.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: worksheet.as_bytes().to_vec(),
            },
            TemplateEntry {
                name: "xl/worksheets/sheet2.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: b"<worksheet/>".to_vec(),
            },
        ];
        replace_collection_placeholders_matching(
            &mut entries,
            Some("xl/worksheets/sheet1.xml"),
            &wrapper,
            FillConfig::new().direction(FillDirection::Horizontal),
        );
        let xml = std::str::from_utf8(&entries[0].bytes).expect("utf-8");
        assert!(xml.contains(">A<"), "A 应横向展开: {xml}");
        assert!(xml.contains(">B<"), "B 应横向展开: {xml}");
        assert!(entries[1].bytes == b"<worksheet/>", "sheet2 不应被修改");
    }

    /// 对应 Java：`forceNewRow(true)` 垂直展开，逐行复制模板行并平移尾部。
    #[test]
    fn expand_vertical_rows_force_new_row_copies_rows_and_shifts_tail() {
        let wrapper = FillWrapper::named(
            "users",
            [
                TemplateData::new().with("name", "Alice"),
                TemplateData::new().with("name", "Bob"),
                TemplateData::new().with("name", "Carol"),
            ],
        );
        let xml = r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>{users.name}</t></is></c></row><row r="2"><c r="A2" t="inlineStr"><is><t>Footer</t></is></c></row></sheetData></worksheet>"#;
        let expanded =
            expand_vertical_rows(xml, &wrapper, FillConfig::new().force_new_row(true), &[])
                .expect("force_new_row 展开必须成功");
        assert!(expanded.contains("Alice"), "{expanded}");
        assert!(expanded.contains("Bob"), "{expanded}");
        assert!(expanded.contains("Carol"), "{expanded}");
        assert!(expanded.contains("Footer"), "{expanded}");
    }

    /// 对应 Java：默认（非 forceNewRow）垂直展开，追加行写入已有行之后。
    #[test]
    fn expand_vertical_rows_without_force_reuses_template_row() {
        let wrapper = FillWrapper::named(
            "users",
            [
                TemplateData::new().with("name", "Alice"),
                TemplateData::new().with("name", "Bob"),
            ],
        );
        let xml = r#"<worksheet><sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>{users.name}</t></is></c></row><row r="2"><c r="B2" t="inlineStr"><is><t>Preserve</t></is></c></row><row r="3"><c r="A3" t="inlineStr"><is><t>Footer</t></is></c></row></sheetData></worksheet>"#;
        let expanded = expand_vertical_rows(xml, &wrapper, FillConfig::new(), &[])
            .expect("默认垂直展开必须成功");
        assert!(expanded.contains("Alice"), "{expanded}");
        assert!(expanded.contains("Bob"), "{expanded}");
        assert!(expanded.contains("Preserve"), "{expanded}");
        assert!(expanded.contains("Footer"), "{expanded}");
    }
}
