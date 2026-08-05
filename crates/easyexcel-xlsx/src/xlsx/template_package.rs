//! 可复用的 OOXML 模板包修改引擎。
//!
//! 本模块负责工作表部件定位、创建、行追加、布局修改和样式表合并，
//! 不依赖 EasyExcel builder、listener、annotation 或门面值类型。

use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use easyexcel_io::{Error, Result};
use zip::CompressionMethod;

use super::ooxml_package::{OoxmlPackage, OoxmlZipEntry};
use super::package::resolve_target;
use super::template_styles::merge_compiled_styles;
use super::template_xml::{
    TemplateCellValue, TemplateMergeRange, append_sparse_rows, apply_column_widths,
    apply_merge_ranges, attribute_value, cell_style_index, escape_xml, worksheet_max_row,
};

const WORKBOOK_PATH: &str = "xl/workbook.xml";
const WORKBOOK_RELS_PATH: &str = "xl/_rels/workbook.xml.rels";
const CONTENT_TYPES_PATH: &str = "[Content_Types].xml";
const STYLES_PATH: &str = "xl/styles.xml";

const EMPTY_WORKSHEET_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
    r#"<dimension ref="A1"/><sheetData></sheetData></worksheet>"#
);

/// 保留未知部件的 OOXML 模板工作簿。
#[derive(Debug, Clone)]
pub struct OoxmlTemplatePackage {
    entries: OoxmlPackage,
}

impl OoxmlTemplatePackage {
    /// 从 XLSX/OOXML 字节载入模板包。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(Self {
            entries: OoxmlPackage::from_bytes(bytes)?,
        })
    }

    /// 使用已载入的 OOXML 条目包构建模板引擎。
    #[must_use]
    pub fn from_package(entries: OoxmlPackage) -> Self {
        Self { entries }
    }

    /// 取出底层无损 OOXML 包。
    #[must_use]
    pub fn into_package(self) -> OoxmlPackage {
        self.entries
    }

    /// 按工作簿顺序返回工作表名称。
    pub fn sheet_names(&self) -> Result<Vec<String>> {
        Ok(self
            .workbook_sheets()?
            .into_iter()
            .map(|(name, _)| name)
            .collect())
    }

    /// 返回指定工作表下一条可追加的零基行号。
    pub fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        let path = self.worksheet_path_by_name(sheet_name)?;
        let xml = self.entry_xml(&path)?;
        let maximum = worksheet_max_row(&xml);
        if maximum == 0 && !xml.contains("<row") {
            Ok(0)
        } else {
            Ok(u32::try_from(maximum.saturating_add(1)).unwrap_or(u32::MAX))
        }
    }

    /// 按工作表名称解析对应的 worksheet part 路径。
    pub fn worksheet_path_by_name(&self, sheet_name: &str) -> Result<String> {
        let sheets = self.workbook_sheets()?;
        let selected = sheets
            .iter()
            .find(|(name, _)| name == sheet_name)
            .ok_or_else(|| Error::SheetNotFound(sheet_name.to_owned()))?;
        self.worksheet_part_for_relationship(&selected.1, &selected.0)
    }

    /// 按零基工作表下标返回名称与 worksheet part 路径。
    pub fn worksheet_path_by_index(&self, index: usize) -> Result<(String, String)> {
        let sheets = self.workbook_sheets()?;
        let selected = sheets
            .get(index)
            .ok_or_else(|| Error::SheetNotFound(index.to_string()))?;
        let path = self.worksheet_part_for_relationship(&selected.1, &selected.0)?;
        Ok((selected.0.clone(), path))
    }

    /// 确保指定名称的工作表存在。
    pub fn ensure_sheet(&mut self, sheet_name: &str) -> Result<()> {
        if self.sheet_names()?.iter().any(|name| name == sheet_name) {
            return Ok(());
        }
        self.create_sheet(sheet_name)
    }

    /// 创建空工作表并继承第一个工作表的默认行高和列宽。
    pub fn create_sheet(&mut self, sheet_name: &str) -> Result<()> {
        let sheet_part = next_worksheet_part_name(&self.entries);
        let workbook_index = entry_index(&self.entries, WORKBOOK_PATH)?;
        let rels_index = entry_index(&self.entries, WORKBOOK_RELS_PATH)?;
        let content_types_index = entry_index(&self.entries, CONTENT_TYPES_PATH)?;

        let workbook_xml = entry_string(&self.entries[workbook_index])?;
        let rels_xml = entry_string(&self.entries[rels_index])?;
        let content_types_xml = entry_string(&self.entries[content_types_index])?;
        let relationship_id = next_relationship_id(&rels_xml);
        let sheet_id = next_sheet_id(&workbook_xml);
        let escaped_name = escape_xml(sheet_name);
        let sheet_tag = format!(
            "<sheet name=\"{escaped_name}\" sheetId=\"{sheet_id}\" r:id=\"{relationship_id}\"/>"
        );
        let relationship_tag = format!(
            "<Relationship Id=\"{relationship_id}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/{}\"/>",
            sheet_part
                .strip_prefix("xl/worksheets/")
                .unwrap_or(sheet_part.as_str())
        );
        let override_tag = format!(
            "<Override PartName=\"/{sheet_part}\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>"
        );

        self.entries[workbook_index].bytes =
            insert_before_close_tag(&workbook_xml, "</sheets>", &sheet_tag)?.into_bytes();
        self.entries[rels_index].bytes =
            insert_before_close_tag(&rels_xml, "</Relationships>", &relationship_tag)?.into_bytes();
        self.entries[content_types_index].bytes =
            insert_before_close_tag(&content_types_xml, "</Types>", &override_tag)?.into_bytes();
        let bytes = blank_worksheet_with_inherited_format(&self.entries);
        self.entries.push(OoxmlZipEntry {
            name: sheet_part,
            is_dir: false,
            compression: CompressionMethod::Deflated,
            unix_mode: None,
            bytes,
        });
        Ok(())
    }

    /// 追加稀疏行、行高和样式索引，并保留显式缺席的 Java `null` 行。
    pub fn append_rows(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, TemplateCellValue)>],
        row_heights: &[Option<u16>],
        cell_styles: &[Vec<Option<u32>>],
        absent_rows: &[bool],
    ) -> Result<u32> {
        if rows.is_empty() {
            return self.next_row_for_sheet(sheet_name);
        }
        validate_row_shapes(rows, row_heights, cell_styles, absent_rows)?;
        let path = self.worksheet_path_by_name(sheet_name)?;
        let entry = self.entry_mut(&path)?;
        let xml = String::from_utf8(std::mem::take(&mut entry.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        let (updated, next_row) =
            append_sparse_rows(&xml, rows, row_heights, cell_styles, absent_rows)?;
        entry.bytes = updated.into_bytes();
        Ok(next_row)
    }

    /// 应用列宽和绝对合并区域。
    pub fn apply_sheet_layout(
        &mut self,
        sheet_name: &str,
        column_widths: &[(u16, u16)],
        merge_ranges: &[TemplateMergeRange],
    ) -> Result<()> {
        if column_widths.is_empty() && merge_ranges.is_empty() {
            return Ok(());
        }
        let path = self.worksheet_path_by_name(sheet_name)?;
        let entry = self.entry_mut(&path)?;
        let mut xml = String::from_utf8(std::mem::take(&mut entry.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        if !column_widths.is_empty() {
            xml = apply_column_widths(&xml, column_widths)?;
        }
        if !merge_ranges.is_empty() {
            xml = apply_merge_ranges(&xml, merge_ranges)?;
        }
        entry.bytes = xml.into_bytes();
        Ok(())
    }

    /// 合并编译工作簿的样式表并返回目标 XF 索引。
    pub fn import_compiled_styles(
        &mut self,
        compiled_xlsx: &[u8],
        style_count: usize,
    ) -> Result<Vec<u32>> {
        if style_count == 0 {
            return Ok(Vec::new());
        }
        let compiled = Self::from_bytes(compiled_xlsx)?;
        let source_styles = compiled.entry_xml(STYLES_PATH)?;
        let (_, source_sheet_path) = compiled.worksheet_path_by_index(0)?;
        let source_sheet = compiled.entry_xml(&source_sheet_path)?;
        let source_indexes = (1..=style_count)
            .map(|row| {
                cell_style_index(&source_sheet, &format!("A{row}")).ok_or_else(|| {
                    Error::Xlsx(format!("compiled style cell A{row} has no style index"))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let destination = self.entry_mut(STYLES_PATH)?;
        let destination_styles = String::from_utf8(std::mem::take(&mut destination.bytes))
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        let (updated, mapped) =
            merge_compiled_styles(&destination_styles, &source_styles, &source_indexes)?;
        destination.bytes = updated.into_bytes();
        Ok(mapped)
    }

    /// 序列化为 XLSX 字节。
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.entries.to_bytes()
    }

    /// 保存到路径。
    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        self.entries.save_to_path(path)
    }

    /// 保存到输出流。
    pub fn save_to_writer(&self, output: &mut dyn Write) -> Result<()> {
        self.entries.save_to_writer(output)
    }

    fn workbook_sheets(&self) -> Result<Vec<(String, String)>> {
        let xml = self.entry_xml(WORKBOOK_PATH)?;
        Ok(xml_elements(&xml, "sheet")
            .filter_map(|element| {
                Some((
                    attribute_value(element, "name")?.to_owned(),
                    attribute_value(element, "r:id")?.to_owned(),
                ))
            })
            .collect())
    }

    fn worksheet_part_for_relationship(
        &self,
        relationship_id: &str,
        sheet_name: &str,
    ) -> Result<String> {
        let xml = self.entry_xml(WORKBOOK_RELS_PATH)?;
        let target = xml_elements(&xml, "Relationship")
            .find(|element| attribute_value(element, "Id") == Some(relationship_id))
            .and_then(|element| attribute_value(element, "Target"))
            .ok_or_else(|| {
                Error::Xlsx(format!(
                    "workbook relationship {relationship_id} for sheet {sheet_name} is missing"
                ))
            })?;
        let normalized = resolve_target(WORKBOOK_PATH, target)?;
        self.entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(&normalized))
            .map(|entry| entry.name.clone())
            .ok_or_else(|| {
                Error::Xlsx(format!(
                    "worksheet part {normalized} for sheet {sheet_name} is missing"
                ))
            })
    }

    fn entry_xml(&self, path: &str) -> Result<String> {
        let entry = self
            .entries
            .iter()
            .find(|entry| entry.name.eq_ignore_ascii_case(path))
            .ok_or_else(|| Error::Xlsx(format!("template does not contain {path}")))?;
        entry_string(entry)
    }

    fn entry_mut(&mut self, path: &str) -> Result<&mut OoxmlZipEntry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.name.eq_ignore_ascii_case(path))
            .ok_or_else(|| Error::Xlsx(format!("template does not contain {path}")))
    }
}

impl Deref for OoxmlTemplatePackage {
    type Target = OoxmlPackage;

    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for OoxmlTemplatePackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

fn validate_row_shapes(
    rows: &[Vec<(usize, TemplateCellValue)>],
    row_heights: &[Option<u16>],
    cell_styles: &[Vec<Option<u32>>],
    absent_rows: &[bool],
) -> Result<()> {
    if !absent_rows.is_empty() && absent_rows.len() != rows.len() {
        return Err(Error::Xlsx(
            "template absent-row count does not match appended row count".to_owned(),
        ));
    }
    if !row_heights.is_empty() && row_heights.len() != rows.len() {
        return Err(Error::Xlsx(
            "template row-height count does not match appended row count".to_owned(),
        ));
    }
    if !cell_styles.is_empty()
        && (cell_styles.len() != rows.len()
            || cell_styles
                .iter()
                .zip(rows)
                .any(|(styles, row)| styles.len() != row.len()))
    {
        return Err(Error::Xlsx(
            "template cell-style shape does not match appended rows".to_owned(),
        ));
    }
    Ok(())
}

fn entry_index(entries: &[OoxmlZipEntry], path: &str) -> Result<usize> {
    entries
        .iter()
        .position(|entry| entry.name.eq_ignore_ascii_case(path))
        .ok_or_else(|| Error::Xlsx(format!("template missing {path}")))
}

fn entry_string(entry: &OoxmlZipEntry) -> Result<String> {
    String::from_utf8(entry.bytes.clone()).map_err(|error| Error::Xlsx(error.to_string()))
}

fn xml_elements<'a>(xml: &'a str, tag: &'a str) -> impl Iterator<Item = &'a str> + 'a {
    let open = format!("<{tag}");
    let mut offset = 0;
    std::iter::from_fn(move || {
        let relative = xml[offset..].find(&open)?;
        let start = offset + relative;
        let end = start + xml[start..].find('>')? + 1;
        offset = end;
        Some(&xml[start..end])
    })
}

fn next_worksheet_part_name(entries: &[OoxmlZipEntry]) -> String {
    let maximum = entries
        .iter()
        .filter_map(|entry| {
            let lower = entry.name.to_ascii_lowercase();
            let rest = lower.strip_prefix("xl/worksheets/sheet")?;
            rest.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<usize>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    format!("xl/worksheets/sheet{}.xml", maximum.saturating_add(1))
}

fn next_relationship_id(xml: &str) -> String {
    format!("rId{}", next_numeric_attribute(xml, "Id=\"rId"))
}

fn next_sheet_id(xml: &str) -> usize {
    xml_elements(xml, "sheet")
        .filter_map(|element| attribute_value(element, "sheetId")?.parse::<usize>().ok())
        .max()
        .unwrap_or(0)
        .saturating_add(1)
}

fn next_numeric_attribute(xml: &str, marker: &str) -> usize {
    let mut maximum = 0;
    let mut offset = 0;
    while let Some(relative) = xml[offset..].find(marker) {
        let start = offset + relative + marker.len();
        let digits = xml[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect::<String>();
        if let Ok(value) = digits.parse::<usize>() {
            maximum = maximum.max(value);
        }
        offset = start;
    }
    maximum.saturating_add(1)
}

fn insert_before_close_tag(xml: &str, close_tag: &str, fragment: &str) -> Result<String> {
    let index = xml
        .find(close_tag)
        .ok_or_else(|| Error::Xlsx(format!("template XML is missing {close_tag}")))?;
    Ok(format!("{}{}{}", &xml[..index], fragment, &xml[index..]))
}

fn blank_worksheet_with_inherited_format(entries: &[OoxmlZipEntry]) -> Vec<u8> {
    let Some(source) = entries.iter().find(|entry| {
        let lower = entry.name.to_ascii_lowercase();
        lower.starts_with("xl/worksheets/sheet") && lower.ends_with(".xml")
    }) else {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    };
    let Ok(xml) = std::str::from_utf8(&source.bytes) else {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    };
    let format = extract_xml_element(xml, "sheetFormatPr").unwrap_or_default();
    let columns = extract_xml_element(xml, "cols").unwrap_or_default();
    if format.is_empty() && columns.is_empty() {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    }
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<dimension ref="A1"/>{format}{columns}<sheetData></sheetData></worksheet>"#
        )
    )
    .into_bytes()
}

fn extract_xml_element(xml: &str, tag: &str) -> Option<String> {
    let start = xml.find(&format!("<{tag}"))?;
    let rest = &xml[start..];
    let close = format!("</{tag}>");
    if let Some(close_at) = rest.find(&close) {
        return Some(rest[..close_at + close.len()].to_owned());
    }
    let self_close = rest.find("/>")?;
    if rest[..self_close].contains('>') {
        return None;
    }
    Some(rest[..=self_close + 1].to_owned())
}

#[cfg(test)]
mod tests {
    use super::{
        EMPTY_WORKSHEET_XML, OoxmlZipEntry, blank_worksheet_with_inherited_format,
        extract_xml_element, insert_before_close_tag, next_relationship_id, next_sheet_id,
        next_worksheet_part_name,
    };
    use zip::CompressionMethod;

    fn worksheet_entry(bytes: Vec<u8>) -> OoxmlZipEntry {
        OoxmlZipEntry {
            name: "xl/worksheets/sheet1.xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes,
        }
    }

    #[test]
    fn blank_worksheet_inherits_format_and_columns() {
        let entries = vec![worksheet_entry(
            br#"<worksheet><sheetFormatPr defaultRowHeight="20"/><cols><col min="1" max="1" width="30" customWidth="1"/></cols><sheetData/></worksheet>"#.to_vec(),
        )];
        let xml = String::from_utf8(blank_worksheet_with_inherited_format(&entries))
            .expect("blank worksheet XML");
        assert!(xml.contains("defaultRowHeight=\"20\""));
        assert!(xml.contains("customWidth=\"1\""));
        assert!(xml.contains("<sheetData>"));
    }

    #[test]
    fn blank_worksheet_falls_back_for_missing_invalid_or_bare_sources() {
        assert_eq!(
            blank_worksheet_with_inherited_format(&[]),
            EMPTY_WORKSHEET_XML.as_bytes()
        );
        assert_eq!(
            blank_worksheet_with_inherited_format(&[worksheet_entry(vec![0xff, 0xfe, 0x00])]),
            EMPTY_WORKSHEET_XML.as_bytes()
        );
        assert_eq!(
            blank_worksheet_with_inherited_format(&[worksheet_entry(
                br"<worksheet><sheetData/></worksheet>".to_vec(),
            )]),
            EMPTY_WORKSHEET_XML.as_bytes()
        );
    }

    #[test]
    fn package_xml_helpers_cover_success_and_error_paths() {
        assert_eq!(
            extract_xml_element(
                "<worksheet><sheetData><row/></sheetData></worksheet>",
                "sheetData",
            ),
            Some("<sheetData><row/></sheetData>".to_owned())
        );
        assert!(extract_xml_element("<sheetData><row/>", "sheetData").is_none());
        assert!(extract_xml_element("<sheetData", "sheetData").is_none());

        let error = insert_before_close_tag("<a/>", "</b>", "x").expect_err("missing tag");
        assert!(error.to_string().contains("missing </b>"));
        assert_eq!(
            insert_before_close_tag("<a></a>", "</a>", "x").expect("insert"),
            "<a>x</a>"
        );
    }

    #[test]
    fn package_identifiers_advance_above_existing_values() {
        let entries = vec![
            worksheet_entry(Vec::new()),
            OoxmlZipEntry {
                name: "xl/worksheets/sheet9.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: Vec::new(),
            },
        ];
        assert_eq!(next_worksheet_part_name(&entries), "xl/worksheets/sheet10.xml");
        assert_eq!(
            next_relationship_id(r#"<Relationship Id="rId2"/><Relationship Id="rId8"/>"#),
            "rId9"
        );
        assert_eq!(
            next_sheet_id(r#"<sheets><sheet sheetId="2"/><sheet sheetId="7"/></sheets>"#),
            8
        );
    }
}
