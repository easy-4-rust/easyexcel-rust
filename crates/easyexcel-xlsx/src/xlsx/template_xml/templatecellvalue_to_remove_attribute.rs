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

#[cfg(test)]
mod tcv_trm_tests {
    use super::*;

    // ── render_cell 覆盖 ──────────────────────────────────────────────

    #[test]
    fn render_cell_empty() {
        let cell = render_cell("A1", &TemplateCellValue::Empty, None);
        assert_eq!(cell, "<c r=\"A1\"></c>");
    }

    #[test]
    fn render_cell_text() {
        let cell = render_cell("B2", &TemplateCellValue::Text("hello".into()), Some(1));
        assert!(cell.contains("t=\"inlineStr\""));
        assert!(cell.contains("s=\"1\""));
        assert!(cell.contains("hello"));
    }

    #[test]
    fn render_cell_text_escapes_special_chars() {
        let cell = render_cell("A1", &TemplateCellValue::Text("<>&\"'".into()), None);
        assert!(cell.contains("&lt;&gt;&amp;&quot;&apos;"));
    }

    #[test]
    fn render_cell_bool_true() {
        let cell = render_cell("A1", &TemplateCellValue::Bool(true), None);
        assert!(cell.contains("t=\"b\""));
        assert!(cell.contains("<v>1</v>"));
    }

    #[test]
    fn render_cell_bool_false() {
        let cell = render_cell("A1", &TemplateCellValue::Bool(false), None);
        assert!(cell.contains("<v>0</v>"));
    }

    #[test]
    fn render_cell_number() {
        let cell = render_cell("A1", &TemplateCellValue::Number("42.5".into()), None);
        assert!(cell.contains("<v>42.5</v>"));
    }

    #[test]
    fn render_cell_date() {
        let cell = render_cell("A1", &TemplateCellValue::Date("2024-01-01".into()), None);
        assert!(cell.contains("t=\"d\""));
        assert!(cell.contains("2024-01-01"));
    }

    #[test]
    fn render_cell_formula() {
        let cell = render_cell("A1", &TemplateCellValue::Formula("SUM(A1:A10)".into()), None);
        assert!(cell.contains("<f>SUM(A1:A10)</f>"));
    }

    #[test]
    fn render_cell_formula_escapes() {
        let cell = render_cell("A1", &TemplateCellValue::Formula("IF(A1>0,\"yes\",\"no\")".into()), None);
        assert!(cell.contains("&gt;"));
    }

    #[test]
    fn render_cell_error() {
        let cell = render_cell("A1", &TemplateCellValue::Error("#VALUE!".into()), None);
        assert!(cell.contains("t=\"e\""));
        assert!(cell.contains("#VALUE!"));
    }

    #[test]
    fn render_cell_rich_text() {
        use crate::xlsx::template_xml::TemplateRichText;
        let rt = TemplateRichText::plain("rich");
        let cell = render_cell("A1", &TemplateCellValue::RichText(rt), None);
        assert!(cell.contains("t=\"inlineStr\""));
        assert!(cell.contains("rich"));
    }

    #[test]
    fn render_cell_with_style() {
        let cell = render_cell("A1", &TemplateCellValue::Number("1".into()), Some(5));
        assert!(cell.contains("s=\"5\""));
    }

    // ── set_cell_value 覆盖 ───────────────────────────────────────────

    #[test]
    fn set_cell_value_updates_existing_cell() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1:A1"/><sheetData><row r="1"><c r="A1"><v>old</v></c></row></sheetData></worksheet>"#;
        let result = set_cell_value(xml, 0, 0, &TemplateCellValue::Number("new".into())).unwrap();
        assert!(result.contains("new"));
        assert!(!result.contains("old"));
    }

    #[test]
    fn set_cell_value_adds_to_existing_row() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1:A1"/><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;
        let result = set_cell_value(xml, 0, 1, &TemplateCellValue::Number("2".into())).unwrap();
        assert!(result.contains("B1"));
        assert!(result.contains("<v>2</v>"));
    }

    #[test]
    fn set_cell_value_creates_new_row_and_cell() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1:A1"/><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;
        let result = set_cell_value(xml, 5, 0, &TemplateCellValue::Text("new".into())).unwrap();
        assert!(result.contains("A6"));
    }

    #[test]
    fn set_cell_value_for_empty_sheet_data() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1"/><sheetData/></worksheet>"#;
        let result = set_cell_value(xml, 0, 0, &TemplateCellValue::Number("1".into())).unwrap();
        assert!(result.contains("A1"));
    }

    // ── apply_sheet_protection 覆盖 ───────────────────────────────────

    #[test]
    fn apply_sheet_protection_inserts_new() {
        let xml = r#"<?xml version="1.0"?><worksheet><sheetData/></worksheet>"#;
        let result = apply_sheet_protection(xml, "pass").unwrap();
        assert!(result.contains("sheetProtection"));
        assert!(result.contains("password="));
    }

    #[test]
    fn apply_sheet_protection_replaces_existing() {
        let xml = r#"<?xml version="1.0"?><worksheet><sheetProtection password="0000" sheet="1" objects="1" scenarios="1"/><sheetData/></worksheet>"#;
        let result = apply_sheet_protection(xml, "newpass").unwrap();
        assert!(result.contains("sheetProtection"));
        // 不应包含旧密码
        let count = result.matches("sheetProtection").count();
        assert_eq!(count, 1);
    }

    #[test]
    fn apply_sheet_protection_inserts_before_worksheet_end_without_sheet_data() {
        let xml = "<?xml version=\"1.0\"?><worksheet></worksheet>";
        let result = apply_sheet_protection(xml, "pass").unwrap();
        assert!(result.contains("sheetProtection"));
    }

    // ── cell_style_index 覆盖 ─────────────────────────────────────────

    #[test]
    fn cell_style_index_finds_style() {
        let xml = r#"<row><c r="A1" s="3"><v>1</v></c></row>"#;
        assert_eq!(cell_style_index(xml, "A1"), Some(3));
    }

    #[test]
    fn cell_style_index_returns_none_for_missing() {
        let xml = r#"<row><c r="A1"><v>1</v></c></row>"#;
        assert_eq!(cell_style_index(xml, "A1"), None);
    }

    #[test]
    fn cell_style_index_returns_none_for_missing_cell() {
        let xml = r#"<row><c r="A1" s="3"><v>1</v></c></row>"#;
        assert_eq!(cell_style_index(xml, "B1"), None);
    }

    // ── worksheet_max_row 覆盖 ────────────────────────────────────────

    #[test]
    fn worksheet_max_row_finds_max() {
        let xml = r#"<sheetData><row r="1"/><row r="5"/><row r="3"/></sheetData>"#;
        assert_eq!(worksheet_max_row(xml), 5);
    }

    #[test]
    fn worksheet_max_row_returns_zero_for_empty() {
        assert_eq!(worksheet_max_row("<sheetData/>"), 0);
    }

    // ── column_name 覆盖 ──────────────────────────────────────────────

    #[test]
    fn column_name_single_letter() {
        assert_eq!(column_name(1), "A");
        assert_eq!(column_name(26), "Z");
    }

    #[test]
    fn column_name_multi_letter() {
        assert_eq!(column_name(27), "AA");
        assert_eq!(column_name(28), "AB");
    }

    #[test]
    fn column_name_empty_for_zero() {
        assert_eq!(column_name(0), "");
    }

    // ── escape_xml 覆盖 ───────────────────────────────────────────────

    #[test]
    fn escape_xml_all_special_chars() {
        assert_eq!(escape_xml("<>&\"'"), "&lt;&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn escape_xml_no_special_chars() {
        assert_eq!(escape_xml("hello world"), "hello world");
    }

    // ── expand_self_closing_sheet_data 覆盖 ───────────────────────────

    #[test]
    fn expand_self_closing_sheet_data_expands() {
        let xml = "<worksheet><sheetData/></worksheet>";
        let result = expand_self_closing_sheet_data(xml).unwrap();
        assert!(result.contains("</sheetData>"));
    }

    #[test]
    fn expand_self_closing_sheet_data_noop_if_already_open() {
        let xml = "<worksheet><sheetData><row/></sheetData></worksheet>";
        let result = expand_self_closing_sheet_data(xml).unwrap();
        assert_eq!(result, xml);
    }

    #[test]
    fn expand_self_closing_sheet_data_error_for_missing() {
        let result = expand_self_closing_sheet_data("<worksheet/>");
        assert!(result.is_err());
    }

    // ── update_worksheet_dimension 覆盖 ───────────────────────────────

    #[test]
    fn update_worksheet_dimension_updates_ref() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1:A1"/><sheetData><row r="1"><c r="A1"><v>1</v></c><c r="C1"><v>2</v></c></row><row r="3"><c r="B3"><v>3</v></c></row></sheetData></worksheet>"#;
        let result = update_worksheet_dimension(xml);
        assert!(result.contains("A1:C3"));
    }

    #[test]
    fn update_worksheet_dimension_noop_for_empty() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1"/><sheetData/></worksheet>"#;
        let result = update_worksheet_dimension(xml);
        assert!(result.contains("A1"));
    }

    // ── parse_cell_reference 覆盖 ─────────────────────────────────────

    #[test]
    fn parse_cell_reference_simple() {
        assert_eq!(parse_cell_reference("A1"), Some((1, 1)));
        assert_eq!(parse_cell_reference("B2"), Some((2, 2)));
    }

    #[test]
    fn parse_cell_reference_with_dollar() {
        assert_eq!(parse_cell_reference("$A$1"), Some((1, 1)));
        assert_eq!(parse_cell_reference("$B$10"), Some((2, 10)));
    }

    #[test]
    fn parse_cell_reference_returns_none_for_invalid() {
        assert_eq!(parse_cell_reference(""), None);
        assert_eq!(parse_cell_reference("123"), None);
        assert_eq!(parse_cell_reference("A0"), None);
    }

    // ── attribute_value 覆盖 ──────────────────────────────────────────

    #[test]
    fn attribute_value_finds_value() {
        let xml = r#"<element name="test" value="123">"#;
        assert_eq!(attribute_value(xml, "name"), Some("test"));
        assert_eq!(attribute_value(xml, "value"), Some("123"));
    }

    #[test]
    fn attribute_value_returns_none_for_missing() {
        let xml = r#"<element name="test">"#;
        assert_eq!(attribute_value(xml, "missing"), None);
    }

    // ── all_cells 覆盖 ────────────────────────────────────────────────

    #[test]
    fn all_cells_finds_multiple_cells() {
        let xml = r#"<row><c r="A1"><v>1</v></c><c r="B1"><v>2</v></c></row>"#;
        let cells = all_cells(xml);
        assert_eq!(cells.len(), 2);
    }

    #[test]
    fn all_cells_handles_self_closing() {
        let xml = r#"<row><c r="A1"/></row>"#;
        let cells = all_cells(xml);
        assert_eq!(cells.len(), 1);
    }

    #[test]
    fn all_cells_empty_for_no_cells() {
        let cells = all_cells("<row/>");
        assert!(cells.is_empty());
    }

    // ── element_value 覆盖 ────────────────────────────────────────────

    #[test]
    fn element_value_finds_text() {
        let xml = "<author>Alice</author>";
        assert_eq!(element_value(xml, "author"), Some("Alice"));
    }

    #[test]
    fn element_value_returns_none_for_missing() {
        assert_eq!(element_value("<other/>", "author"), None);
    }

    // ── replace_attribute 覆盖 ────────────────────────────────────────

    #[test]
    fn replace_attribute_updates() {
        let xml = r#"<element name="old">"#;
        let result = replace_attribute(xml, "name", "new");
        assert_eq!(result, r#"<element name="new">"#);
    }

    #[test]
    fn replace_attribute_noop_for_missing() {
        let xml = "<element>";
        let result = replace_attribute(xml, "missing", "value");
        assert_eq!(result, xml);
    }

    // ── replace_tag_attribute 覆盖 ────────────────────────────────────

    #[test]
    fn replace_tag_attribute_updates() {
        let xml = r#"<dimension ref="A1:A1"/>"#;
        let result = replace_tag_attribute(xml, "dimension", "ref", "A1:B2");
        assert!(result.contains("A1:B2"));
    }

    #[test]
    fn replace_tag_attribute_noop_for_missing_tag() {
        let xml = "<worksheet/>";
        let result = replace_tag_attribute(xml, "dimension", "ref", "A1:B2");
        assert_eq!(result, xml);
    }

    // ── remove_attribute 覆盖 ─────────────────────────────────────────

    #[test]
    fn remove_attribute_removes() {
        let xml = r#"<element name="test" id="1">"#;
        let result = remove_attribute(xml, "name");
        assert!(!result.contains("name="));
        assert!(result.contains("id=\"1\""));
    }

    #[test]
    fn remove_attribute_noop_for_missing() {
        let xml = r#"<element id="1">"#;
        let result = remove_attribute(xml, "name");
        assert_eq!(result, xml);
    }

    // ── append_sparse_rows 覆盖 ───────────────────────────────────────

    #[test]
    fn append_sparse_rows_adds_to_empty_sheet() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1"/><sheetData/></worksheet>"#;
        let rows = vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let (result, next) = append_sparse_rows(xml, &rows, &[], &[], &[]).unwrap();
        assert_eq!(next, 2);
        assert!(result.contains("<row r=\"1\">"));
    }

    #[test]
    fn append_sparse_rows_adds_to_existing() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1"/><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;
        let rows = vec![vec![(0, TemplateCellValue::Number("2".into()))]];
        let (result, next) = append_sparse_rows(xml, &rows, &[], &[], &[]).unwrap();
        assert_eq!(next, 3);
        assert!(result.contains("<row r=\"2\">"));
    }

    #[test]
    fn append_sparse_rows_with_height() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1"/><sheetData/></worksheet>"#;
        let rows = vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let heights = vec![Some(30)];
        let (result, _) = append_sparse_rows(xml, &rows, &heights, &[], &[]).unwrap();
        assert!(result.contains("ht=\"30\""));
        assert!(result.contains("customHeight=\"1\""));
    }

    #[test]
    fn append_sparse_rows_with_absent() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1"/><sheetData/></worksheet>"#;
        let rows = vec![
            vec![(0, TemplateCellValue::Number("1".into()))],
            vec![(0, TemplateCellValue::Number("2".into()))],
        ];
        let absent = vec![false, true];
        let (result, next) = append_sparse_rows(xml, &rows, &[], &[], &absent).unwrap();
        assert_eq!(next, 3);
        // 只有一行被追加
        assert!(result.contains("<row r=\"1\">"));
    }

    #[test]
    fn append_sparse_rows_with_styles() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1"/><sheetData/></worksheet>"#;
        let rows = vec![vec![(0, TemplateCellValue::Number("1".into()))]];
        let styles = vec![vec![Some(3)]];
        let (result, _) = append_sparse_rows(xml, &rows, &[], &styles, &[]).unwrap();
        assert!(result.contains("s=\"3\""));
    }

    // ── apply_column_widths 覆盖 ──────────────────────────────────────

    #[test]
    fn apply_column_widths_inserts_before_sheet_data() {
        let xml = "<worksheet><sheetData/></worksheet>";
        let result = apply_column_widths(xml, &[(0, 20)]).unwrap();
        assert!(result.contains("<cols>"));
        assert!(result.contains("width=\"20\""));
    }

    #[test]
    fn apply_column_widths_appends_to_existing_cols() {
        let xml = r#"<worksheet><cols><col min="1" max="1" width="10" customWidth="1"/></cols><sheetData/></worksheet>"#;
        let result = apply_column_widths(xml, &[(1, 30)]).unwrap();
        assert!(result.contains("width=\"30\""));
    }

    #[test]
    fn apply_column_widths_replaces_self_closing_cols() {
        let xml = "<worksheet><cols/><sheetData/></worksheet>";
        let result = apply_column_widths(xml, &[(0, 15)]).unwrap();
        assert!(result.contains("<cols>"));
        assert!(result.contains("width=\"15\""));
    }

    // ── apply_merge_ranges 覆盖 ───────────────────────────────────────

    #[test]
    fn apply_merge_ranges_inserts_new() {
        // 使用一个不会与 dimension ref 重复的合并范围
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1:A1"/><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData></worksheet>"#;
        let ranges = vec![TemplateMergeRange { first_row: 0, first_column: 0, last_row: 2, last_column: 2 }];
        let result = apply_merge_ranges(xml, &ranges).unwrap();
        assert!(result.contains("<mergeCells"));
        assert!(result.contains("A1:C3"));
    }

    #[test]
    fn apply_merge_ranges_appends_to_existing() {
        let xml = r#"<?xml version="1.0"?><worksheet><dimension ref="A1:C3"/><sheetData><row r="1"><c r="A1"><v>1</v></c></row></sheetData><mergeCells count="1"><mergeCell ref="A1:B2"/></mergeCells></worksheet>"#;
        let ranges = vec![TemplateMergeRange { first_row: 2, first_column: 0, last_row: 2, last_column: 2 }];
        let result = apply_merge_ranges(xml, &ranges).unwrap();
        assert!(result.contains("A3:C3"));
        assert!(result.contains("count=\"2\""));
    }

    #[test]
    fn apply_merge_ranges_noop_if_already_present() {
        let xml = r#"<?xml version="1.0"?><worksheet><sheetData/><mergeCells count="1"><mergeCell ref="A1:B2"/></mergeCells></worksheet>"#;
        let ranges = vec![TemplateMergeRange { first_row: 0, first_column: 0, last_row: 1, last_column: 1 }];
        let result = apply_merge_ranges(xml, &ranges).unwrap();
        assert_eq!(result, xml);
    }

    // ── TemplateCellValue::as_text 覆盖 ───────────────────────────────

    #[test]
    fn as_text_all_variants() {
        assert_eq!(TemplateCellValue::Empty.as_text(), "");
        assert_eq!(TemplateCellValue::Text("hi".into()).as_text(), "hi");
        assert_eq!(TemplateCellValue::Bool(true).as_text(), "true");
        assert_eq!(TemplateCellValue::Number("42".into()).as_text(), "42");
        assert_eq!(TemplateCellValue::Date("2024-01".into()).as_text(), "2024-01");
        assert_eq!(TemplateCellValue::Formula("SUM".into()).as_text(), "SUM");
        assert_eq!(TemplateCellValue::Error("#ERR".into()).as_text(), "#ERR");
    }
}
