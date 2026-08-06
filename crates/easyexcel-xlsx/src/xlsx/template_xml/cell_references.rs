/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回 XML 片段内全部 A1 单元格属性引用及其字节区间。
#[must_use]
pub fn cell_references(xml: &str) -> Vec<(usize, usize, &str)> {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按行列增量平移单个 A1 引用。
#[must_use]
pub fn shift_cell_reference(reference: &str, row_delta: usize, column_delta: usize) -> String {
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
    format!("{}{}", column_name(column + column_delta), row + row_delta)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移一个工作表行 XML 片段及其单元格引用。
#[must_use]
pub fn shift_row(xml: &str, row_delta: usize, column_delta: usize) -> String {
    let mut shifted = xml.to_owned();
    for reference in cell_references(xml).into_iter().rev() {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移 XML 中全部完整 `<row>...</row>` 元素。
#[must_use]
pub fn shift_rows(xml: &str, delta: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 判断标记是否存在且未被奇数个反斜杠转义。
#[must_use]
pub fn contains_unescaped(value: &str, marker: &str) -> bool {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回 `<row>` 标签的一基行号。
#[must_use]
pub fn row_index(row: &str) -> Option<usize> {
    attribute_value(row, "r")?.parse().ok()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移工作表合并、链接、筛选、校验、条件格式与公式引用。
#[must_use]
pub fn shift_worksheet_metadata(xml: &str, threshold_row: usize, delta: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移指定 XML 标签中的引用属性。
#[must_use]
pub fn shift_tag_references(
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移空格分隔的单元格或区域引用列表。
#[must_use]
pub fn shift_reference_list(value: &str, threshold_row: usize, delta: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移所有 `<f>` 公式元素内的 A1 引用。
#[must_use]
pub fn shift_formula_elements(xml: &str, threshold_row: usize, delta: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在公式文本中识别并平移独立 A1 引用。
#[must_use]
pub fn shift_formula_references(formula: &str, threshold_row: usize, delta: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移单个 A1 引用，同时保留绝对列/行标记。
#[must_use]
pub fn shift_a1_reference(reference: &str, threshold_row: usize, delta: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 确保克隆行标签带有显式一基行号。
#[must_use]
pub fn row_tag_with_reference(row_tag: &str, row: usize) -> String {
    if attribute_value(row_tag, "r").is_some() {
        row_tag.to_owned()
    } else {
        replace_attribute(row_tag, "r", &row.to_string())
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 校验集合填充目标是否位于 XLSX 工作表范围内。
///
/// # Errors
///
/// 行列越界时返回 XLSX 格式错误。
pub fn validate_collection_target(row: usize, column: usize) -> Result<()> {
    if row >= 1_048_576 {
        return Err(Error::Xlsx(
            "collection fill row exceeds XLSX limit".to_owned(),
        ));
    }
    if column >= 16_384 {
        return Err(Error::Xlsx(
            "collection fill column exceeds XLSX limit".to_owned(),
        ));
    }
    Ok(())
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回工作表最后一个完整行元素的零基行号。
#[must_use]
pub fn last_worksheet_row(xml: &str) -> Option<usize> {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 平移指定零基行之后的工作表行及关联元数据。
#[must_use]
pub fn shift_worksheet_rows_after(xml: &str, row: usize, delta: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 sharedStrings.xml 中全部 `<si>` 文本。
#[must_use]
pub fn shared_string_values(xml: &str) -> Vec<String> {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 合并 XML 片段内全部 `<t>` 文本节点。
#[must_use]
pub fn text_node_values(xml: &str) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在目标一基行合并、插入或追加集合填充行。
#[must_use]
pub fn upsert_collection_row(xml: &str, collection_row: &str, target_row: usize) -> String {
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 按单元格引用合并两个同一行号的 OOXML 行片段。
#[must_use]
pub fn merge_collection_cells(existing_row: &str, collection_row: &str) -> String {
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

#[cfg(test)]
#[path = "../template_xml_tests/tests.rs"]
mod tests;
