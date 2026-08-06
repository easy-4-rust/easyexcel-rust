//! XLSX 模板来源识别、工作表选择和 OOXML part 定位。
//!
//! 本模块只处理路径、魔数、工作簿关系和中立工作表选择，不依赖
//! EasyExcel builder、listener、annotation 或门面错误类型。

use std::path::Path;

use easyexcel_io::{Error, Result, path_has_extension};

use super::ooxml_package::OoxmlZipEntry;
use super::package::{normalize_path, resolve_target};
use super::template_xml::attribute_value;

const WORKBOOK_PATH: &str = "xl/workbook.xml";
const WORKBOOK_RELS_PATH: &str = "xl/_rels/workbook.xml.rels";

/// 中立工作表选择器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateSheetSelector<'a> {
    /// 第一张工作表。
    First,
    /// 零基工作表下标。
    Index(usize),
    /// 工作表名称。
    Name(&'a str),
}

impl<'a> TemplateSheetSelector<'a> {
    /// 判断两个模板工作表选择器是否指向同一逻辑目标。
    ///
    /// 未显式选择与零基下标 `0` 都表示第一张工作表；名称只与相同名称等价。
    #[must_use]
    pub fn equivalent<'b>(self, other: TemplateSheetSelector<'b>) -> bool {
        match (self, other) {
            (
                TemplateSheetSelector::First | TemplateSheetSelector::Index(0),
                TemplateSheetSelector::First | TemplateSheetSelector::Index(0),
            ) => true,
            (TemplateSheetSelector::Index(left), TemplateSheetSelector::Index(right)) => {
                left == right
            }
            (TemplateSheetSelector::Name(left), TemplateSheetSelector::Name(right)) => {
                left == right
            }
            _ => false,
        }
    }
}

/// 返回是否配置了模板文件或模板字节。
#[must_use]
pub const fn has_template(template_file: Option<&Path>, template_bytes: Option<&[u8]>) -> bool {
    template_file.is_some() || template_bytes.is_some()
}

/// 从文件或内存字节载入模板内容。
pub fn load_template_bytes(
    template_file: Option<&Path>,
    template_bytes: Option<&[u8]>,
) -> Result<Vec<u8>> {
    if let Some(bytes) = template_bytes {
        return Ok(bytes.to_vec());
    }
    if let Some(path) = template_file {
        return Ok(std::fs::read(path)?);
    }
    Err(Error::Unsupported(
        "with_template requires a template file or template bytes".to_owned(),
    ))
}

/// 校验用于 XLSX 生成路径的模板来源。
///
/// CSV 不是工作簿模板；OLE/BIFF8 模板必须交给 XLS 引擎处理。
pub fn validate_xlsx_template_source(
    template_file: Option<&Path>,
    template_bytes: Option<&[u8]>,
) -> Result<()> {
    if let Some(path) = template_file {
        if path_has_extension(path, "csv") {
            return Err(Error::Unsupported("csv cannot use template.".to_owned()));
        }
        if path_has_extension(path, "xls") {
            return Err(Error::Unsupported(
                "legacy XLS template cannot seed an XLSX workbook; write to a .xls path instead"
                    .to_owned(),
            ));
        }
    }
    if let Some(bytes) = template_bytes {
        if looks_like_csv(bytes) {
            return Err(Error::Unsupported("csv cannot use template.".to_owned()));
        }
        if easyexcel_io::looks_like_cfb(bytes) {
            return Err(Error::Unsupported(
                "legacy XLS template cannot seed an XLSX workbook; write to a .xls path instead"
                    .to_owned(),
            ));
        }
    }
    Ok(())
}

/// 按 Java `sheet(index)` / `sheet(name)` 选择规则解析目标工作表。
#[must_use]
pub fn resolve_sheet_target(
    sheet_names: &[String],
    sheet_index: Option<usize>,
    sheet_name: &str,
) -> (usize, String, bool) {
    if let Some(index) = sheet_index {
        if let Some(name) = sheet_names.get(index) {
            return (index, name.clone(), false);
        }
        return (index, sheet_name.to_owned(), true);
    }
    if let Some((index, name)) = sheet_names
        .iter()
        .enumerate()
        .find(|(_, name)| *name == sheet_name)
    {
        return (index, name.clone(), false);
    }
    (sheet_names.len(), sheet_name.to_owned(), true)
}

/// 从工作簿条目解析选择工作表对应的 worksheet part 路径。
pub fn worksheet_path(
    entries: &[OoxmlZipEntry],
    selector: TemplateSheetSelector<'_>,
) -> Result<String> {
    let workbook = find_entry(entries, WORKBOOK_PATH);
    let relationships = find_entry(entries, WORKBOOK_RELS_PATH);
    if let (Some(workbook), Some(relationships)) = (workbook, relationships) {
        let workbook = entry_text(workbook)?;
        let relationships = entry_text(relationships)?;
        let sheets = workbook_sheets(workbook);
        let selected = match selector {
            TemplateSheetSelector::First => sheets.first(),
            TemplateSheetSelector::Index(index) => sheets.get(index),
            TemplateSheetSelector::Name(name) => {
                sheets.iter().find(|(sheet_name, _)| sheet_name == name)
            }
        }
        .ok_or_else(|| Error::SheetNotFound(selector_label(selector)))?;
        let target = workbook_relationship_target(relationships, &selected.1).ok_or_else(|| {
            Error::Xlsx(format!(
                "workbook relationship {} for sheet {} is missing",
                selected.1, selected.0
            ))
        })?;
        let normalized = normalize_workbook_target(target)?;
        return find_entry(entries, &normalized)
            .map(|entry| entry.name.clone())
            .ok_or_else(|| {
                Error::Xlsx(format!(
                    "worksheet part {normalized} for sheet {} is missing",
                    selected.0
                ))
            });
    }

    let worksheets = entries
        .iter()
        .filter(|entry| {
            entry.name.starts_with("xl/worksheets/")
                && Path::new(&entry.name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("xml"))
        })
        .collect::<Vec<_>>();
    let index = match selector {
        TemplateSheetSelector::First => 0,
        TemplateSheetSelector::Index(index) => index,
        TemplateSheetSelector::Name(name) => {
            return Err(Error::SheetNotFound(name.to_owned()));
        }
    };
    worksheets
        .get(index)
        .map(|entry| entry.name.clone())
        .ok_or_else(|| Error::SheetNotFound(selector_label(selector)))
}

/// 提取工作簿中按顺序声明的 `(名称, relationship id)`。
#[must_use]
pub fn workbook_sheets(xml: &str) -> Vec<(String, String)> {
    xml_elements(xml, "sheet")
        .filter_map(|element| {
            Some((
                attribute_value(element, "name")?.to_owned(),
                attribute_value(element, "r:id")?.to_owned(),
            ))
        })
        .collect()
}

/// 将 workbook relationship 的 Target 解析为包内绝对路径。
pub fn normalize_workbook_target(target: &str) -> Result<String> {
    if target.starts_with("xl/") {
        normalize_path(target)
    } else {
        resolve_target(WORKBOOK_PATH, target)
    }
}

fn looks_like_csv(bytes: &[u8]) -> bool {
    if super::looks_like_zip(bytes) || easyexcel_io::looks_like_cfb(bytes) {
        return false;
    }
    bytes
        .iter()
        .take(64)
        .all(|byte| byte.is_ascii_whitespace() || byte.is_ascii_graphic() || *byte == b'\t')
}

fn find_entry<'a>(entries: &'a [OoxmlZipEntry], path: &str) -> Option<&'a OoxmlZipEntry> {
    entries
        .iter()
        .find(|entry| entry.name.eq_ignore_ascii_case(path))
}

fn entry_text(entry: &OoxmlZipEntry) -> Result<&str> {
    std::str::from_utf8(&entry.bytes).map_err(|error| Error::Xlsx(error.to_string()))
}

fn workbook_relationship_target<'a>(xml: &'a str, relationship_id: &str) -> Option<&'a str> {
    xml_elements(xml, "Relationship")
        .find(|element| attribute_value(element, "Id") == Some(relationship_id))
        .and_then(|element| attribute_value(element, "Target"))
}

/// 迭代指定名称的 XML 起始标签。
///
/// 该轻量扫描器用于已知 OOXML 元数据标签，不执行通用 XML 解析。
pub fn xml_elements<'a>(xml: &'a str, name: &'a str) -> impl Iterator<Item = &'a str> {
    let marker = format!("<{name}");
    let mut offset = 0;
    std::iter::from_fn(move || {
        while let Some(relative_start) = xml[offset..].find(&marker) {
            let start = offset + relative_start;
            let after_name = start + marker.len();
            if xml
                .as_bytes()
                .get(after_name)
                .is_some_and(u8::is_ascii_alphanumeric)
            {
                offset = after_name;
                continue;
            }
            let end = start + xml[start..].find('>')? + 1;
            offset = end;
            return Some(&xml[start..end]);
        }
        None
    })
}

fn selector_label(selector: TemplateSheetSelector<'_>) -> String {
    match selector {
        TemplateSheetSelector::First => "0".to_owned(),
        TemplateSheetSelector::Index(index) => index.to_string(),
        TemplateSheetSelector::Name(name) => name.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{TemplateSheetSelector, normalize_workbook_target, xml_elements};

    #[test]
    fn first_and_zero_index_are_equivalent_template_targets() {
        assert!(TemplateSheetSelector::First.equivalent(TemplateSheetSelector::Index(0)));
        assert!(TemplateSheetSelector::Index(2).equivalent(TemplateSheetSelector::Index(2)));
        assert!(
            TemplateSheetSelector::Name("Data").equivalent(TemplateSheetSelector::Name("Data"))
        );
        assert!(!TemplateSheetSelector::First.equivalent(TemplateSheetSelector::Index(1)));
        assert!(
            !TemplateSheetSelector::Name("Data").equivalent(TemplateSheetSelector::Name("Other"))
        );
    }

    #[test]
    fn workbook_targets_are_resolved_from_the_xl_workbook_part() {
        assert_eq!(
            normalize_workbook_target("xl/worksheets/sheet1.xml").expect("absolute target"),
            "xl/worksheets/sheet1.xml"
        );
        assert_eq!(
            normalize_workbook_target("worksheets/sheet1.xml").expect("relative target"),
            "xl/worksheets/sheet1.xml"
        );
        assert_eq!(
            normalize_workbook_target("/xl/styles.xml").expect("root target"),
            "xl/styles.xml"
        );
    }

    #[test]
    fn xml_element_scanner_matches_complete_tag_names_only() {
        let xml = r#"<worksheet><row r="1"/><row r="2"/><rowBreaks/></worksheet>"#;
        let rows = xml_elements(xml, "row").collect::<Vec<_>>();
        assert_eq!(rows.len(), 2);
        assert!(xml_elements(xml, "missing").next().is_none());
    }
}
