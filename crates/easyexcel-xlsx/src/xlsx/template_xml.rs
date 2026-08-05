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
    let mut maximum_row = 1usize;
    let mut maximum_column = 1usize;
    let mut remaining = xml;
    while let Some(start) = remaining.find("<c ") {
        remaining = &remaining[start + 3..];
        let Some(end) = remaining.find('>') else {
            break;
        };
        if let Some(reference) = attribute_value(&remaining[..end], "r")
            && let Some((column, row)) = parse_cell_reference(reference)
        {
            maximum_row = maximum_row.max(row);
            maximum_column = maximum_column.max(column);
        }
        remaining = &remaining[end + 1..];
    }
    let reference = format!("A1:{}{maximum_row}", column_name(maximum_column));
    if let Some(start) = xml.find("<dimension")
        && let Some(relative_end) = xml[start..].find("/>")
    {
        let end = start + relative_end + 2;
        return format!(
            "{}<dimension ref=\"{reference}\"/>{}",
            &xml[..start],
            &xml[end..]
        );
    }
    xml.to_owned()
}

/// 解析 A1 引用为一基 `(column, row)`。
#[must_use]
pub fn parse_cell_reference(reference: &str) -> Option<(usize, usize)> {
    let split = reference.find(|character: char| character.is_ascii_digit())?;
    let (column, row) = reference.split_at(split);
    let row = row.parse::<usize>().ok()?;
    let mut column_index = 0usize;
    for byte in column.bytes() {
        if !byte.is_ascii_alphabetic() {
            return None;
        }
        column_index = column_index
            .saturating_mul(26)
            .saturating_add(usize::from(byte.to_ascii_uppercase() - b'A' + 1));
    }
    Some((column_index, row))
}

/// 从 XML 开始标签片段读取双引号属性值。
#[must_use]
pub fn attribute_value<'a>(xml: &'a str, attribute: &str) -> Option<&'a str> {
    let marker = format!(" {attribute}=\"");
    let (_, value) = xml.split_once(&marker)?;
    value.split_once('"').map(|(value, _)| value)
}
