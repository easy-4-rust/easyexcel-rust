//! XLSX 模板工作表 XML 修改原语。
//!
//! 输入使用中立值和坐标，不依赖 EasyExcel builder、handler 或 annotation。

use std::fmt::Write as _;

use easyexcel_io::{Error, Result};

/// 可直接写入 SpreadsheetML 单元格的中立值。
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateCellValue {
    /// 空单元格。
    Empty,
    /// 内联字符串。
    Text(String),
    /// 布尔值。
    Bool(bool),
    /// 已验证的数字词法值。
    Number(String),
    /// ISO 8601 日期或日期时间。
    Date(String),
    /// 不含外层 `<f>` 的公式表达式。
    Formula(String),
    /// Excel 错误文本。
    Error(String),
}

impl TemplateCellValue {
    /// 返回适合占位符字符串替换的显示文本。
    #[must_use]
    pub fn as_text(&self) -> String {
        match self {
            Self::Empty => String::new(),
            Self::Text(value)
            | Self::Number(value)
            | Self::Date(value)
            | Self::Formula(value)
            | Self::Error(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
        }
    }
}

/// 工作表绝对合并区域。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TemplateMergeRange {
    /// 首行，零基。
    pub first_row: u32,
    /// 末行，零基且包含。
    pub last_row: u32,
    /// 首列，零基。
    pub first_column: u16,
    /// 末列，零基且包含。
    pub last_column: u16,
}

/// 在 `sheetData` 末尾追加稀疏行并更新 dimension。
pub fn append_sparse_rows(
    xml: &str,
    rows: &[Vec<(usize, TemplateCellValue)>],
    row_heights: &[Option<u16>],
    cell_styles: &[Vec<Option<u32>>],
    absent_rows: &[bool],
) -> Result<(String, u32)> {
    let xml = expand_self_closing_sheet_data(xml)?;
    let sheet_data_end = xml
        .find("</sheetData>")
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    let maximum = worksheet_max_row(&xml[..sheet_data_end]);
    let next_row = if maximum == 0 && !xml[..sheet_data_end].contains("<row") {
        1usize
    } else {
        maximum.saturating_add(1)
    };
    let mut appended = String::new();
    for (row_offset, values) in rows.iter().enumerate() {
        let row_index = next_row + row_offset;
        if absent_rows.get(row_offset).copied().unwrap_or(false) {
            continue;
        }
        if let Some(height) = row_heights.get(row_offset).copied().flatten() {
            let _ = write!(
                appended,
                "<row r=\"{row_index}\" ht=\"{height}\" customHeight=\"1\">"
            );
        } else {
            let _ = write!(appended, "<row r=\"{row_index}\">");
        }
        for (cell_offset, (physical_index, value)) in values.iter().enumerate() {
            let reference = format!("{}{row_index}", column_name(physical_index + 1));
            let style = cell_styles
                .get(row_offset)
                .and_then(|styles| styles.get(cell_offset))
                .copied()
                .flatten();
            appended.push_str(&render_cell(&reference, value, style));
        }
        appended.push_str("</row>");
    }
    let expanded = format!(
        "{}{}{}",
        &xml[..sheet_data_end],
        appended,
        &xml[sheet_data_end..]
    );
    let next = u32::try_from(next_row.saturating_add(rows.len())).unwrap_or(u32::MAX);
    Ok((update_worksheet_dimension(&expanded), next))
}

/// 添加或更新工作表列宽定义。
pub fn apply_column_widths(xml: &str, widths: &[(u16, u16)]) -> Result<String> {
    let mut tags = String::new();
    for (column, width) in widths {
        let one_based = u32::from(*column) + 1;
        let _ = write!(
            tags,
            "<col min=\"{one_based}\" max=\"{one_based}\" width=\"{width}\" customWidth=\"1\"/>"
        );
    }
    if let Some(end) = xml.find("</cols>") {
        return Ok(format!("{}{}{}", &xml[..end], tags, &xml[end..]));
    }
    if let Some(start) = xml.find("<cols") {
        let relative_end = xml[start..]
            .find("/>")
            .ok_or_else(|| Error::Xlsx("worksheet contains malformed cols element".to_owned()))?;
        let end = start + relative_end + 2;
        return Ok(format!(
            "{}<cols>{}</cols>{}",
            &xml[..start],
            tags,
            &xml[end..]
        ));
    }
    let insertion = xml
        .find("<sheetData")
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    Ok(format!(
        "{}<cols>{}</cols>{}",
        &xml[..insertion],
        tags,
        &xml[insertion..]
    ))
}

/// 添加工作表合并区域并维护 `count`。
pub fn apply_merge_ranges(xml: &str, ranges: &[TemplateMergeRange]) -> Result<String> {
    let refs = ranges
        .iter()
        .map(|range| {
            format!(
                "{}{}:{}{}",
                column_name(usize::from(range.first_column) + 1),
                range.first_row + 1,
                column_name(usize::from(range.last_column) + 1),
                range.last_row + 1
            )
        })
        .filter(|reference| !xml.contains(&format!("ref=\"{reference}\"")))
        .collect::<Vec<_>>();
    if refs.is_empty() {
        return Ok(xml.to_owned());
    }
    let tags = refs
        .iter()
        .map(|reference| format!("<mergeCell ref=\"{reference}\"/>"))
        .collect::<String>();
    if let Some(start) = xml.find("<mergeCells") {
        let tag_end = start
            + xml[start..]
                .find('>')
                .ok_or_else(|| Error::Xlsx("malformed mergeCells element".to_owned()))?;
        let close = xml[tag_end + 1..]
            .find("</mergeCells>")
            .map(|offset| tag_end + 1 + offset)
            .ok_or_else(|| Error::Xlsx("malformed mergeCells element".to_owned()))?;
        let current_count = attribute_value(&xml[start..=tag_end], "count")
            .and_then(|count| count.parse::<usize>().ok())
            .unwrap_or_else(|| xml[tag_end + 1..close].matches("<mergeCell").count());
        let new_count = current_count.saturating_add(refs.len());
        let mut updated = xml.to_owned();
        if let Some(count) = attribute_value(&xml[start..=tag_end], "count") {
            updated = updated.replacen(
                &format!(" count=\"{count}\""),
                &format!(" count=\"{new_count}\""),
                1,
            );
        }
        let close = updated
            .find("</mergeCells>")
            .ok_or_else(|| Error::Xlsx("malformed mergeCells element".to_owned()))?;
        return Ok(format!(
            "{}{}{}",
            &updated[..close],
            tags,
            &updated[close..]
        ));
    }
    let insertion = xml
        .find("</sheetData>")
        .map(|index| index + "</sheetData>".len())
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    Ok(format!(
        "{}<mergeCells count=\"{}\">{}</mergeCells>{}",
        &xml[..insertion],
        refs.len(),
        tags,
        &xml[insertion..]
    ))
}

/// 渲染一个 SpreadsheetML 单元格。
#[must_use]
pub fn render_cell(reference: &str, value: &TemplateCellValue, style: Option<u32>) -> String {
    let style_attribute = style
        .map(|index| format!(" s=\"{index}\""))
        .unwrap_or_default();
    let start = format!("<c r=\"{reference}\"{style_attribute}>");
    match value {
        TemplateCellValue::Empty => format!("{start}</c>"),
        TemplateCellValue::Text(text) => format!(
            "<c r=\"{reference}\"{style_attribute} t=\"inlineStr\"><is><t>{}</t></is></c>",
            escape_xml(text)
        ),
        TemplateCellValue::Bool(flag) => format!(
            "<c r=\"{reference}\"{style_attribute} t=\"b\"><v>{}</v></c>",
            u8::from(*flag)
        ),
        TemplateCellValue::Number(number) => format!("{start}<v>{number}</v></c>"),
        TemplateCellValue::Date(value) => {
            format!("<c r=\"{reference}\"{style_attribute} t=\"d\"><v>{value}</v></c>")
        }
        TemplateCellValue::Formula(formula) => {
            format!("{start}<f>{}</f></c>", escape_xml(formula))
        }
        TemplateCellValue::Error(error) => format!(
            "<c r=\"{reference}\"{style_attribute} t=\"e\"><v>{}</v></c>",
            escape_xml(error)
        ),
    }
}

/// 查找既有单元格的样式索引。
#[must_use]
pub fn cell_style_index(sheet_xml: &str, reference: &str) -> Option<usize> {
    let marker = format!("<c r=\"{reference}\"");
    let (_, cell) = sheet_xml.split_once(&marker)?;
    let tag = cell.split_once('>')?.0;
    attribute_value(tag, "s")?.parse().ok()
}

/// 返回工作表中最大的显式一基行号。
#[must_use]
pub fn worksheet_max_row(xml: &str) -> usize {
    let mut maximum = 0usize;
    let mut remaining = xml;
    while let Some(start) = remaining.find("<row") {
        remaining = &remaining[start + 4..];
        let Some(end) = remaining.find('>') else {
            break;
        };
        if let Some(index) =
            attribute_value(&remaining[..end], "r").and_then(|value| value.parse::<usize>().ok())
        {
            maximum = maximum.max(index);
        }
        remaining = &remaining[end + 1..];
    }
    maximum
}

/// 将一基列号转换为 Excel 列名。
#[must_use]
pub fn column_name(mut column: usize) -> String {
    let mut name = String::new();
    while column > 0 {
        let remainder = (column - 1) % 26;
        name.insert(0, char::from(b'A' + u8::try_from(remainder).unwrap_or(0)));
        column = (column - 1) / 26;
    }
    name
}

/// XML 转义文本。
#[must_use]
pub fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// 展开自闭合 `sheetData` 元素。
pub fn expand_self_closing_sheet_data(xml: &str) -> Result<String> {
    if xml.contains("</sheetData>") {
        return Ok(xml.to_owned());
    }
    let start = xml
        .find("<sheetData")
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    let after = &xml[start..];
    let relative_end = after
        .find("/>")
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    if after[..relative_end].contains('>') {
        return Err(Error::Xlsx(
            "worksheet does not contain sheetData".to_owned(),
        ));
    }
    let end = start + relative_end;
    Ok(format!(
        "{}{}></sheetData>{}",
        &xml[..start],
        &xml[start..end],
        &xml[end + 2..]
    ))
}

/// 根据显式单元格引用更新 worksheet dimension。
#[must_use]
pub fn update_worksheet_dimension(xml: &str) -> String {
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

/// 解析 A1 引用为一基 `(column, row)`。
#[must_use]
pub fn parse_cell_reference(reference: &str) -> Option<(usize, usize)> {
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

/// 从 XML 开始标签片段读取双引号属性值。
#[must_use]
pub fn attribute_value<'a>(xml: &'a str, attribute: &str) -> Option<&'a str> {
    let marker = format!(" {attribute}=\"");
    let (_, value) = xml.split_once(&marker)?;
    value.split_once('"').map(|(value, _)| value)
}

/// 返回 XML 片段内全部完整单元格元素及字节区间。
#[must_use]
pub fn all_cells(xml: &str) -> Vec<(usize, usize, &str)> {
    let mut cells = Vec::new();
    let mut offset = 0;
    while let Some((start, end)) = find_next_cell(xml, offset) {
        cells.push((start, end, &xml[start..end]));
        offset = end;
    }
    cells
}

fn find_next_cell(xml: &str, from: usize) -> Option<(usize, usize)> {
    let bytes = xml.as_bytes();
    let mut search = from;
    while search < xml.len() {
        let relative = xml[search..].find("<c")?;
        let start = search + relative;
        let after_c = start + 2;
        let next = *bytes.get(after_c)?;
        if !matches!(next, b' ' | b'\t' | b'\n' | b'\r' | b'/' | b'>') {
            search = after_c;
            continue;
        }
        let relative_gt = xml[after_c..].find('>')?;
        let gt = after_c + relative_gt;
        if gt > start && bytes[gt - 1] == b'/' {
            return Some((start, gt + 1));
        }
        let relative_end = xml[gt..].find("</c>")?;
        return Some((start, gt + relative_end + 4));
    }
    None
}

/// 返回简单 XML 元素的文本内容。
#[must_use]
pub fn element_value<'a>(xml: &'a str, element: &str) -> Option<&'a str> {
    let start_marker = format!("<{element}>");
    let end_marker = format!("</{element}>");
    let start = xml.find(&start_marker)? + start_marker.len();
    let end = start + xml[start..].find(&end_marker)?;
    Some(&xml[start..end])
}

/// 替换开始标签中的双引号属性值；属性不存在时保持原文。
#[must_use]
pub fn replace_attribute(xml: &str, attribute: &str, value: &str) -> String {
    let Some(current) = attribute_value(xml, attribute) else {
        return xml.to_owned();
    };
    xml.replacen(
        &format!(" {attribute}=\"{current}\""),
        &format!(" {attribute}=\"{value}\""),
        1,
    )
}

/// 替换指定开始标签中的属性值。
#[must_use]
pub fn replace_tag_attribute(xml: &str, tag: &str, attribute: &str, value: &str) -> String {
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

/// 删除开始标签中的双引号属性；属性不存在时保持原文。
#[must_use]
pub fn remove_attribute(xml: &str, attribute: &str) -> String {
    let Some(current) = attribute_value(xml, attribute) else {
        return xml.to_owned();
    };
    xml.replacen(&format!(" {attribute}=\"{current}\""), "", 1)
}

/// 返回 XML 片段内全部 A1 单元格属性引用及其字节区间。
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

/// 按行列增量平移单个 A1 引用。
#[must_use]
pub fn shift_cell_reference(
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
    format!("{}{}", column_name(column + column_delta), row + row_delta)
}

/// 平移一个工作表行 XML 片段及其单元格引用。
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

/// 平移 XML 中全部完整 `<row>...</row>` 元素。
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

/// 判断标记是否存在且未被奇数个反斜杠转义。
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

/// 返回 `<row>` 标签的一基行号。
#[must_use]
pub fn row_index(row: &str) -> Option<usize> {
    attribute_value(row, "r")?.parse().ok()
}

/// 平移工作表合并、链接、筛选、校验、条件格式与公式引用。
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

/// 平移指定 XML 标签中的引用属性。
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

/// 平移空格分隔的单元格或区域引用列表。
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

/// 平移所有 `<f>` 公式元素内的 A1 引用。
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

/// 在公式文本中识别并平移独立 A1 引用。
#[must_use]
pub fn shift_formula_references(
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

/// 平移单个 A1 引用，同时保留绝对列/行标记。
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

/// 确保克隆行标签带有显式一基行号。
#[must_use]
pub fn row_tag_with_reference(row_tag: &str, row: usize) -> String {
    if attribute_value(row_tag, "r").is_some() {
        row_tag.to_owned()
    } else {
        replace_attribute(row_tag, "r", &row.to_string())
    }
}

/// 校验集合填充目标是否位于 XLSX 工作表范围内。
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

/// 返回工作表最后一个完整行元素的零基行号。
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

/// 平移指定零基行之后的工作表行及关联元数据。
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

/// 解析 sharedStrings.xml 中全部 `<si>` 文本。
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

/// 合并 XML 片段内全部 `<t>` 文本节点。
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

/// 在目标一基行合并、插入或追加集合填充行。
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

/// 按单元格引用合并两个同一行号的 OOXML 行片段。
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
mod tests {
    use super::{
        attribute_value, cell_style_index, column_name, escape_xml, parse_cell_reference,
        row_index, update_worksheet_dimension, worksheet_max_row,
    };

    #[test]
    fn column_names_cover_single_and_multiple_letter_ranges() {
        assert_eq!(column_name(1), "A");
        assert_eq!(column_name(26), "Z");
        assert_eq!(column_name(27), "AA");
        assert_eq!(column_name(52), "AZ");
        assert_eq!(column_name(53), "BA");
        assert_eq!(column_name(703), "AAA");
    }

    #[test]
    fn xml_escape_covers_all_special_characters() {
        assert_eq!(escape_xml(""), "");
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("<tag>&\"'"), "&lt;tag&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn cell_reference_parser_covers_valid_invalid_and_bounds() {
        assert_eq!(parse_cell_reference("A1"), Some((1, 1)));
        assert_eq!(parse_cell_reference("AB10"), Some((28, 10)));
        assert_eq!(parse_cell_reference("$XFD$1048576"), Some((16_384, 1_048_576)));
        assert_eq!(parse_cell_reference(""), None);
        assert_eq!(parse_cell_reference("1A"), None);
        assert_eq!(parse_cell_reference("A!1"), None);
        assert_eq!(parse_cell_reference("XFE1"), None);
    }

    #[test]
    fn worksheet_helpers_read_attributes_rows_styles_and_dimensions() {
        assert_eq!(attribute_value(r#"<tag attr="value">"#, "attr"), Some("value"));
        assert_eq!(attribute_value(r#"<tag attr="value">"#, "missing"), None);
        assert_eq!(row_index("row"), None);
        assert_eq!(row_index("row r=\"15\""), Some(15));
        assert_eq!(worksheet_max_row(r#"<row r="5"/><row r="10"/>"#), 10);
        assert_eq!(worksheet_max_row("<row"), 0);
        assert_eq!(cell_style_index(r#"<c r="A1" s="5"/>"#, "A1"), Some(5));
        assert_eq!(cell_style_index(r#"<c r="A1"/>"#, "A1"), None);

        let xml = r#"<worksheet><dimension ref="A1"/><sheetData><row r="1"><c r="A1"/></row><row r="5"><c r="C5"/></row></sheetData></worksheet>"#;
        assert!(update_worksheet_dimension(xml).contains("ref=\"A1:C5\""));
        assert_eq!(
            update_worksheet_dimension("<c r=\"A1\"><v>1</v></c>"),
            "<c r=\"A1\"><v>1</v></c>"
        );
    }
}
