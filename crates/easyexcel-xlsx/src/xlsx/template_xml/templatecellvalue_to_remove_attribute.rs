include!("templatecellvalue_to_remove_attribute/template_cell_value.rs");



include!("templatecellvalue_to_remove_attribute/template_merge_range.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 在 `sheetData` 末尾追加稀疏行并更新 dimension。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 添加或更新工作表列宽定义。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 添加工作表合并区域并维护 `count`。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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
    let mut tags = String::new();
    for reference in &refs {
        let _ = write!(tags, "<mergeCell ref=\"{reference}\"/>");
    }
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 渲染一个 `SpreadsheetML` 单元格。
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
        TemplateCellValue::RichText(value) => format!(
            "<c r=\"{reference}\"{style_attribute} t=\"inlineStr\">{}</c>",
            value.inline_string_xml()
        ),
        TemplateCellValue::Comment { value, .. }
        | TemplateCellValue::Hyperlink { value, .. }
        | TemplateCellValue::Images { value, .. } => render_cell(reference, value, style),
    }
}

/// 在工作表 XML 中新增或替换一个绝对坐标单元格，并保留原有样式索引。
///
/// # Errors
///
/// 工作表缺少 `sheetData`、坐标越界或 XML 结构不完整时返回错误。
pub fn set_cell_value(
    xml: &str,
    zero_based_row: u32,
    zero_based_column: u16,
    value: &TemplateCellValue,
) -> Result<String> {
    let one_based_row = usize::try_from(zero_based_row)
        .unwrap_or(usize::MAX)
        .checked_add(1)
        .ok_or_else(|| Error::Xlsx("worksheet row index overflow".to_owned()))?;
    let one_based_column = usize::from(zero_based_column) + 1;
    let reference = format!("{}{one_based_row}", column_name(one_based_column));

    for (start, end, cell) in all_cells(xml) {
        if attribute_value(cell, "r") == Some(reference.as_str()) {
            let style = attribute_value(cell, "s").and_then(|value| value.parse::<u32>().ok());
            let replacement = render_cell(&reference, value, style);
            return Ok(update_worksheet_dimension(&format!(
                "{}{}{}",
                &xml[..start],
                replacement,
                &xml[end..]
            )));
        }
    }

    let row_marker = format!(" r=\"{one_based_row}\"");
    let mut search = 0;
    while let Some(relative) = xml[search..].find("<row") {
        let row_start = search + relative;
        let row_open_end = xml[row_start..]
            .find('>')
            .map(|offset| row_start + offset + 1)
            .ok_or_else(|| Error::Xlsx("worksheet row start tag is incomplete".to_owned()))?;
        let row_open = &xml[row_start..row_open_end];
        if row_open.contains(&row_marker) {
            let row_end = xml[row_open_end..]
                .find("</row>")
                .map(|offset| row_open_end + offset)
                .ok_or_else(|| Error::Xlsx("worksheet row end tag is missing".to_owned()))?;
            let replacement = render_cell(&reference, value, None);
            return Ok(update_worksheet_dimension(&format!(
                "{}{}{}",
                &xml[..row_end],
                replacement,
                &xml[row_end..]
            )));
        }
        search = row_open_end;
    }

    let expanded = expand_self_closing_sheet_data(xml)?;
    let sheet_data_end = expanded
        .find("</sheetData>")
        .ok_or_else(|| Error::Xlsx("worksheet does not contain sheetData".to_owned()))?;
    let row = format!(
        "<row r=\"{one_based_row}\">{}</row>",
        render_cell(&reference, value, None)
    );
    Ok(update_worksheet_dimension(&format!(
        "{}{}{}",
        &expanded[..sheet_data_end],
        row,
        &expanded[sheet_data_end..]
    )))
}

/// 添加或更新 OOXML 工作表保护元素。
///
/// # Errors
///
/// 工作表 XML 缺少根结束标签时返回错误。
pub fn apply_sheet_protection(xml: &str, password: &str) -> Result<String> {
    let hash = legacy_password_hash(password);
    let protection = format!(
        "<sheetProtection password=\"{hash:04X}\" sheet=\"1\" objects=\"1\" scenarios=\"1\"/>"
    );
    if let Some(start) = xml.find("<sheetProtection") {
        let end = xml[start..]
            .find('>')
            .map(|offset| start + offset + 1)
            .ok_or_else(|| Error::Xlsx("sheetProtection tag is incomplete".to_owned()))?;
        return Ok(format!("{}{}{}", &xml[..start], protection, &xml[end..]));
    }
    if let Some(sheet_data_end) = xml.find("</sheetData>") {
        let insert = sheet_data_end + "</sheetData>".len();
        return Ok(format!("{}{}{}", &xml[..insert], protection, &xml[insert..]));
    }
    let end = xml
        .rfind("</worksheet>")
        .ok_or_else(|| Error::Xlsx("worksheet end tag is missing".to_owned()))?;
    Ok(format!("{}{}{}", &xml[..end], protection, &xml[end..]))
}

fn legacy_password_hash(password: &str) -> u16 {
    let utf16: Vec<u16> = password.encode_utf16().collect();
    let mut hash = 0_u16;
    for value in utf16.iter().rev() {
        hash = hash.rotate_left(1) ^ *value;
    }
    hash ^= u16::try_from(utf16.len()).unwrap_or(u16::MAX);
    hash ^ 0xCE4B
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 查找既有单元格的样式索引。
#[must_use]
pub fn cell_style_index(sheet_xml: &str, reference: &str) -> Option<usize> {
    let marker = format!("<c r=\"{reference}\"");
    let (_, cell) = sheet_xml.split_once(&marker)?;
    let tag = cell.split_once('>')?.0;
    attribute_value(tag, "s")?.parse().ok()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回工作表中最大的显式一基行号。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将一基列号转换为 Excel 列名。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 XML 转义文本。
#[must_use]
pub fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 展开自闭合 `sheetData` 元素。
///
/// # Errors
///
/// 底层 OOXML、ZIP、XML 或目标 I/O 操作失败，或输入不符合格式约束时返回错误。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 根据显式单元格引用更新 worksheet dimension。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解析 A1 引用为一基 `(column, row)`。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 从 XML 开始标签片段读取双引号属性值。
#[must_use]
pub fn attribute_value<'a>(xml: &'a str, attribute: &str) -> Option<&'a str> {
    let marker = format!(" {attribute}=\"");
    let (_, value) = xml.split_once(&marker)?;
    value.split_once('"').map(|(value, _)| value)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回 XML 片段内全部完整单元格元素及字节区间。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回简单 XML 元素的文本内容。
#[must_use]
pub fn element_value<'a>(xml: &'a str, element: &str) -> Option<&'a str> {
    let start_marker = format!("<{element}>");
    let end_marker = format!("</{element}>");
    let start = xml.find(&start_marker)? + start_marker.len();
    let end = start + xml[start..].find(&end_marker)?;
    Some(&xml[start..end])
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 替换开始标签中的双引号属性值；属性不存在时保持原文。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 替换指定开始标签中的属性值。
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 删除开始标签中的双引号属性；属性不存在时保持原文。
#[must_use]
pub fn remove_attribute(xml: &str, attribute: &str) -> String {
    let Some(current) = attribute_value(xml, attribute) else {
        return xml.to_owned();
    };
    xml.replacen(&format!(" {attribute}=\"{current}\""), "", 1)
}
