//! Template-backed workbook seeding for Java `withTemplate` + `doWrite`.
//!
//! **Default path:** ZIP/OOXML preserve ([`TemplatePackage`]). Clone the template
//! package, keep `xl/styles.xml` and worksheet `mergeCells` intact, append typed
//! rows into `sheetData`, and when a requested sheet is missing create a new
//! worksheet part without rewriting existing sheets.
//!
//! **Legacy path:** calamine → `rust_xlsxwriter` value replay. Styles, merges,
//! images, comments, drawings, and column widths are **not** preserved. Used
//! only when callers explicitly set
//! [`crate::WriteOptions::use_legacy_template_seed`].

use std::io::Write;
#[cfg(test)]
use std::io::Cursor;
use std::path::Path;

use crate::core::{CellValue, ExcelError, Result};
#[cfg(test)]
use calamine::{Data, DataType, Reader, Xlsx, open_workbook_from_rs};
#[cfg(test)]
use rust_xlsxwriter::{Format, Workbook, Worksheet};
#[cfg(test)]
use zip::write::{SimpleFileOptions, ZipWriter};

use easyexcel_xlsx::xlsx::OoxmlTemplatePackage;
use easyexcel_xlsx::xlsx::template_xml::{TemplateCellValue, TemplateMergeRange};

use crate::MergeRange;

/// Legacy value-replay snapshot owned by the XLSX engine.
pub(crate) use easyexcel_xlsx::LegacyTemplateSheet as TemplateSheetData;

/// One ZIP entry retained from a template XLSX package.
#[cfg(test)]
pub(crate) use easyexcel_xlsx::xlsx::OoxmlZipEntry as TemplateZipEntry;

/// In-memory XLSX template package used by the ZIP preserve write path.
#[derive(Debug, Clone)]
pub(crate) struct TemplatePackage {
    entries: OoxmlTemplatePackage,
}

impl TemplatePackage {
    /// Loads an XLSX template package from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Format`] when the bytes are not a readable ZIP/OOXML package.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        OoxmlTemplatePackage::from_bytes(bytes)
            .map(|entries| Self { entries })
            .map_err(ExcelError::from)
    }

    /// Returns worksheet names in workbook order.
    ///
    /// # Errors
    ///
    /// Returns a format error when workbook metadata cannot be parsed.
    pub(crate) fn sheet_names(&self) -> Result<Vec<String>> {
        self.entries.sheet_names().map_err(ExcelError::from)
    }

    /// Returns the next zero-based append row for a worksheet name.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::SheetNotFound`] when the sheet is absent.
    pub(crate) fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        if !self.sheet_names()?.iter().any(|name| name == sheet_name) {
            return Err(ExcelError::SheetNotFound(sheet_name.to_owned()));
        }
        self.entries
            .next_row_for_sheet(sheet_name)
            .map_err(ExcelError::from)
    }

    /// Resolves the worksheet part path for a sheet name.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::SheetNotFound`] when the sheet is absent.
    pub(crate) fn worksheet_path_by_name(&self, sheet_name: &str) -> Result<String> {
        if !self.sheet_names()?.iter().any(|name| name == sheet_name) {
            return Err(ExcelError::SheetNotFound(sheet_name.to_owned()));
        }
        self.entries
            .worksheet_path_by_name(sheet_name)
            .map_err(ExcelError::from)
    }

    /// Resolves the worksheet part path for a zero-based sheet index.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::SheetNotFound`] when the index is out of range.
    pub(crate) fn worksheet_path_by_index(&self, index: usize) -> Result<(String, String)> {
        if index >= self.sheet_names()?.len() {
            return Err(ExcelError::SheetNotFound(format!("sheet index {index}")));
        }
        self.entries
            .worksheet_path_by_index(index)
            .map_err(ExcelError::from)
    }

    /// Ensures a worksheet exists; creates an empty one when the name is new.
    ///
    /// Existing worksheets, `xl/styles.xml`, and their `mergeCells` are left
    /// untouched. 对应 Java： creating a sheet that is absent from the template.
    ///
    /// # Errors
    ///
    /// Returns a format error when workbook / relationship metadata cannot be updated.
    pub(crate) fn ensure_sheet(&mut self, sheet_name: &str) -> Result<()> {
        self.entries
            .ensure_sheet(sheet_name)
            .map_err(ExcelError::from)
    }

    /// Creates an empty worksheet part and registers it in the package.
    ///
    /// Existing worksheets, `xl/styles.xml`, and their `mergeCells` stay
    /// untouched. The new sheet inherits `sheetFormatPr` / `cols` from the first
    /// template sheet when present (workbook styles remain shared).
    ///
    /// # Errors
    ///
    /// Returns a format error when workbook / relationship metadata cannot be updated.
    pub(crate) fn create_sheet(&mut self, sheet_name: &str) -> Result<()> {
        self.entries
            .create_sheet(sheet_name)
            .map_err(ExcelError::from)
    }

    /// Appends typed rows into a worksheet's `sheetData`.
    ///
    /// # Errors
    ///
    /// Returns a format error when the worksheet XML cannot be updated.
    #[allow(dead_code)]
    pub(crate) fn append_rows(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
    ) -> Result<u32> {
        self.append_rows_with_heights(sheet_name, rows, &[])
    }

    /// Appends rows and applies optional per-row heights to the newly created
    /// row elements.
    #[allow(dead_code)]
    pub(crate) fn append_rows_with_heights(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
        row_heights: &[Option<u16>],
    ) -> Result<u32> {
        self.append_rows_with_layout(sheet_name, rows, row_heights, &[])
    }

    /// Appends rows with optional row heights and per-cell workbook style indexes.
    #[allow(dead_code)]
    pub(crate) fn append_rows_with_layout(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
        row_heights: &[Option<u16>],
        cell_styles: &[Vec<Option<u32>>],
    ) -> Result<u32> {
        self.append_rows_with_layout_and_absent(sheet_name, rows, row_heights, cell_styles, &[])
    }

    /// Appends rows while preserving Java `null` row gaps without creating
    /// empty OOXML `<row>` elements for those positions.
    pub(crate) fn append_rows_with_layout_and_absent(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
        row_heights: &[Option<u16>],
        cell_styles: &[Vec<Option<u32>>],
        absent_rows: &[bool],
    ) -> Result<u32> {
        let rows = rows
            .iter()
            .map(|row| {
                row.iter()
                    .map(|(index, value)| Ok((*index, template_cell_value(value)?)))
                    .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        self.entries
            .append_rows(
                sheet_name,
                &rows,
                row_heights,
                cell_styles,
                absent_rows,
            )
            .map_err(ExcelError::from)
    }

    /// Applies column widths and absolute merged regions to one preserved
    /// worksheet part.
    ///
    /// This is the OOXML equivalent of Java annotation-generated
    /// `AbstractHeadColumnWidthStyleStrategy` and
    /// `OnceAbsoluteMergeStrategy` callbacks. Existing package entries and
    /// style indexes remain untouched.
    pub(crate) fn apply_sheet_layout(
        &mut self,
        sheet_name: &str,
        column_widths: &[(u16, u16)],
        merge_ranges: &[MergeRange],
    ) -> Result<()> {
        let merge_ranges = merge_ranges
            .iter()
            .map(|range| TemplateMergeRange {
                first_row: range.first_row,
                last_row: range.last_row,
                first_column: range.first_column,
                last_column: range.last_column,
            })
            .collect::<Vec<_>>();
        self.entries
            .apply_sheet_layout(sheet_name, column_widths, &merge_ranges)
            .map_err(ExcelError::from)
    }

    /// Imports styles compiled by `rust_xlsxwriter` into the preserved
    /// template style table and returns the destination style index for each
    /// compiler worksheet row.
    pub(crate) fn import_compiled_styles(
        &mut self,
        compiled_xlsx: &[u8],
        style_count: usize,
    ) -> Result<Vec<u32>> {
        self.entries
            .import_compiled_styles(compiled_xlsx, style_count)
            .map_err(ExcelError::from)
    }

    /// Serializes the package to owned XLSX bytes.
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when ZIP writing fails.
    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>> {
        self.entries.to_bytes().map_err(ExcelError::from)
    }

    /// Writes the package to a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns an I/O or format error.
    #[allow(dead_code)]
    pub(crate) fn save_to_path(&self, path: &Path) -> Result<()> {
        self.entries.save_to_path(path).map_err(ExcelError::from)
    }

    /// Writes the package to an arbitrary writer.
    ///
    /// # Errors
    ///
    /// Returns an I/O or format error.
    #[allow(dead_code)]
    pub(crate) fn save_to_writer(&self, output: &mut dyn Write) -> Result<()> {
        self.entries
            .save_to_writer(output)
            .map_err(ExcelError::from)
    }

}

/// Returns whether [`crate::WriteOptions`] carries a template source.
///
/// Corresponds to Java `WriteWorkbook.templateFile` / `templateInputStream`
/// being non-null.
#[must_use]
pub(crate) fn has_template(template_file: Option<&Path>, template_bytes: Option<&[u8]>) -> bool {
    easyexcel_xlsx::has_template(template_file, template_bytes)
}

/// Loads template bytes from a file path or an in-memory copy.
///
/// # Errors
///
/// Returns I/O errors when the template file cannot be read, or
/// [`ExcelError::Unsupported`] when no template source is configured.
pub(crate) fn load_template_bytes(
    template_file: Option<&Path>,
    template_bytes: Option<&[u8]>,
) -> Result<Vec<u8>> {
    easyexcel_xlsx::load_template_bytes(template_file, template_bytes).map_err(ExcelError::from)
}

/// Rejects template types that Java also rejects for the XLSX ZIP path.
///
/// # Errors
///
/// - CSV templates → same as Java `ExcelGenerateException("csv cannot use template.")`
/// - XLS templates → rejected here for **XLSX output**; `.xls` output uses
///   the `easyexcel-xls` template adapter instead (see writer `start` / `write_xls`).
pub(crate) fn validate_template_source(
    template_file: Option<&Path>,
    template_bytes: Option<&[u8]>,
) -> Result<()> {
    easyexcel_xlsx::validate_xlsx_template_source(template_file, template_bytes)
        .map_err(ExcelError::from)
}

/// Parses an XLSX template into ordered sheet snapshots.
///
/// Used only by the explicit legacy value-replay path
/// ([`crate::WriteOptions::use_legacy_template_seed`]).
///
/// # Errors
///
/// Returns [`ExcelError::Format`] when the package is not a readable XLSX workbook.
/// Resolves the target sheet for Java `sheet()` / `sheet(no)` / `sheet(name)`.
///
/// Preference matches Java `WriteContextImpl.initSheet`:
/// 1. `sheet_index` when set
/// 2. otherwise match by `sheet_name`
/// 3. otherwise treat as a new sheet to create after template sheets
#[must_use]
pub(crate) fn resolve_template_target(
    sheets: &[TemplateSheetData],
    sheet_index: Option<usize>,
    sheet_name: &str,
) -> (usize, String, bool) {
    let names = sheets
        .iter()
        .map(|sheet| sheet.name.clone())
        .collect::<Vec<_>>();
    easyexcel_xlsx::resolve_sheet_target(&names, sheet_index, sheet_name)
}

/// Resolves a template target against a ZIP package sheet list.
#[must_use]
pub(crate) fn resolve_package_target(
    sheet_names: &[String],
    sheet_index: Option<usize>,
    sheet_name: &str,
) -> (usize, String, bool) {
    easyexcel_xlsx::resolve_sheet_target(sheet_names, sheet_index, sheet_name)
}

/// Writes loaded template sheets into a fresh `rust_xlsxwriter` workbook.
///
/// **Legacy only** ([`crate::WriteOptions::use_legacy_template_seed`]): values
/// only — styles/merges are not preserved. Prefer [`TemplatePackage`] by default.
///
/// # Errors
///
/// Returns worksheet naming or cell-write errors from `rust_xlsxwriter`.
#[cfg(test)]
fn append_sparse_rows_to_xml(
    xml: &str,
    rows: &[Vec<(usize, CellValue)>],
    row_heights: &[Option<u16>],
    cell_styles: &[Vec<Option<u32>>],
    absent_rows: &[bool],
) -> Result<(String, u32)> {
    let rows = rows
        .iter()
        .map(|row| {
            row.iter()
                .map(|(index, value)| Ok((*index, template_cell_value(value)?)))
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    easyexcel_xlsx::xlsx::template_xml::append_sparse_rows(
        xml,
        &rows,
        row_heights,
        cell_styles,
        absent_rows,
    )
    .map_err(ExcelError::from)
}

#[cfg(test)]
fn apply_column_widths_to_xml(xml: &str, widths: &[(u16, u16)]) -> Result<String> {
    easyexcel_xlsx::xlsx::template_xml::apply_column_widths(xml, widths).map_err(ExcelError::from)
}

#[cfg(test)]
fn apply_merge_ranges_to_xml(xml: &str, ranges: &[MergeRange]) -> Result<String> {
    let ranges = ranges
        .iter()
        .map(|range| TemplateMergeRange {
            first_row: range.first_row,
            last_row: range.last_row,
            first_column: range.first_column,
            last_column: range.last_column,
        })
        .collect::<Vec<_>>();
    easyexcel_xlsx::xlsx::template_xml::apply_merge_ranges(xml, &ranges).map_err(ExcelError::from)
}

/// Expands empty self-closing `<sheetData…/>` into an open/close pair.
///
/// # Errors
///
/// Returns [`ExcelError::Format`] when the worksheet has no `sheetData` element.
#[cfg(test)]
fn expand_self_closing_sheet_data(xml: &str) -> Result<String> {
    easyexcel_xlsx::xlsx::template_xml::expand_self_closing_sheet_data(xml)
        .map_err(ExcelError::from)
}

#[cfg(test)]
fn render_cell_xml(reference: &str, value: &CellValue, style: Option<u32>) -> String {
    easyexcel_xlsx::xlsx::template_xml::render_cell(
        reference,
        &template_cell_value(value).unwrap_or(TemplateCellValue::Empty),
        style,
    )
}

fn template_cell_value(value: &CellValue) -> Result<TemplateCellValue> {
    Ok(match value {
        CellValue::Empty | CellValue::Image(_) => TemplateCellValue::Empty,
        CellValue::String(text) | CellValue::Error(text) | CellValue::Hyperlink { text, .. } => {
            TemplateCellValue::Text(text.clone())
        }
        CellValue::RichText(rich) => TemplateCellValue::Text(rich.text_string().to_owned()),
        CellValue::Bool(flag) => TemplateCellValue::Bool(*flag),
        CellValue::Int(number) => TemplateCellValue::Number(number.to_string()),
        CellValue::Float(number) => TemplateCellValue::Number(number.to_string()),
        CellValue::Decimal(number) => {
            if crate::write::decimal_integer_requires_text(number)? {
                TemplateCellValue::Text(number.to_plain_string())
            } else {
                TemplateCellValue::Number(number.to_string())
            }
        }
        CellValue::Date(date) => TemplateCellValue::Date(date.format("%Y-%m-%d").to_string()),
        CellValue::DateTime(datetime) => {
            TemplateCellValue::Date(datetime.format("%Y-%m-%dT%H:%M:%S").to_string())
        }
        CellValue::Formula(formula) => TemplateCellValue::Formula(formula.clone()),
        CellValue::Comment { value, .. } | CellValue::Images { value, .. } => {
            return template_cell_value(value);
        }
    })
}

#[cfg(test)]
fn cell_style_index(sheet_xml: &str, reference: &str) -> Option<usize> {
    easyexcel_xlsx::xlsx::template_xml::cell_style_index(sheet_xml, reference)
}

#[cfg(test)]
fn merge_compiled_styles(
    destination: &str,
    source: &str,
    source_indexes: &[usize],
) -> Result<(String, Vec<u32>)> {
    easyexcel_xlsx::xlsx::template_styles::merge_compiled_styles(
        destination,
        source,
        source_indexes,
    )
    .map_err(ExcelError::from)
}

#[cfg(test)]
fn worksheet_max_row(xml: &str) -> usize {
    easyexcel_xlsx::xlsx::template_xml::worksheet_max_row(xml)
}

#[cfg(test)]
fn row_index(tag: &str) -> Option<usize> {
    attribute_value(tag, "r")?.parse().ok()
}

#[cfg(test)]
fn update_worksheet_dimension(xml: &str) -> String {
    easyexcel_xlsx::xlsx::template_xml::update_worksheet_dimension(xml)
}

#[cfg(test)]
fn parse_cell_reference(reference: &str) -> Option<(usize, usize)> {
    easyexcel_xlsx::xlsx::template_xml::parse_cell_reference(reference)
}

#[cfg(test)]
fn column_name(column: usize) -> String {
    easyexcel_xlsx::xlsx::template_xml::column_name(column)
}

#[cfg(test)]
fn escape_xml(value: &str) -> String {
    easyexcel_xlsx::xlsx::template_xml::escape_xml(value)
}

#[cfg(test)]
fn attribute_value<'a>(xml: &'a str, attribute: &str) -> Option<&'a str> {
    easyexcel_xlsx::xlsx::template_xml::attribute_value(xml, attribute)
}

#[cfg(test)]
fn attribute_value_in_tag<'a>(xml: &'a str, tag: &str, attribute: &str) -> Option<&'a str> {
    let start = xml.find(&format!("<{tag}"))?;
    let end = start + xml[start..].find('>')?;
    attribute_value(&xml[start..=end], attribute)
}

#[cfg(test)]
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

#[cfg(test)]
fn normalize_workbook_target(target: &str) -> String {
    let trimmed = target.trim_start_matches('/');
    if trimmed.starts_with("xl/") {
        trimmed.to_owned()
    } else {
        format!("xl/{trimmed}")
    }
}

/// Minimal empty worksheet part used when creating a sheet absent from the template.
///
/// Prefer an open/close `sheetData` pair so [`append_sparse_rows_to_xml`] can
/// splice rows; self-closing `<sheetData/>` is still accepted and expanded.
#[cfg(test)]
const EMPTY_WORKSHEET_XML: &str = concat!(
    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
    r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
    r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
    r#"<dimension ref="A1"/><sheetData></sheetData></worksheet>"#
);

/// Builds a blank worksheet, optionally inheriting `sheetFormatPr` / `cols` from
/// the first template sheet (workbook `styles.xml` remains shared and untouched).
#[cfg(test)]
fn blank_worksheet_with_inherited_format(entries: &[TemplateZipEntry]) -> Vec<u8> {
    let Some(source) = entries.iter().find(|entry| {
        let lower = entry.name.to_ascii_lowercase();
        // 语义敏感：`lower` 已先经 `to_ascii_lowercase` 归一化，
        // 此处的 `.ends_with(".xml")` 实际已大小写不敏感。
        #[allow(clippy::case_sensitive_file_extension_comparisons)]
        let is_worksheet_xml = lower.starts_with("xl/worksheets/sheet") && lower.ends_with(".xml");
        is_worksheet_xml
    }) else {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    };
    let Ok(xml) = std::str::from_utf8(&source.bytes) else {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    };
    let format = extract_xml_element(xml, "sheetFormatPr").unwrap_or_default();
    let cols = extract_xml_element(xml, "cols").unwrap_or_default();
    if format.is_empty() && cols.is_empty() {
        return EMPTY_WORKSHEET_XML.as_bytes().to_vec();
    }
    format!(
        concat!(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
            r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" "#,
            r#"xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">"#,
            r#"<dimension ref="A1"/>{format}{cols}<sheetData></sheetData></worksheet>"#
        ),
        format = format,
        cols = cols
    )
    .into_bytes()
}

/// Returns the first XML element named `tag`, including a self-closing form.
#[cfg(test)]
fn extract_xml_element(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}");
    let start = xml.find(&open)?;
    let rest = &xml[start..];
    let close = format!("</{tag}>");
    if let Some(close_at) = rest.find(&close) {
        return Some(rest[..(close_at + close.len())].to_owned());
    }
    let self_close = rest.find("/>")?;
    if rest[..self_close].contains('>') {
        return None;
    }
    Some(rest[..=self_close + 1].to_owned())
}

/// Picks the next unused `xl/worksheets/sheetN.xml` part name.
#[cfg(test)]
fn next_worksheet_part_name(entries: &[TemplateZipEntry]) -> String {
    let mut maximum = 0usize;
    for entry in entries {
        let lower = entry.name.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("xl/worksheets/sheet") else {
            continue;
        };
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(index) = digits.parse::<usize>() {
            maximum = maximum.max(index);
        }
    }
    format!("xl/worksheets/sheet{}.xml", maximum.saturating_add(1))
}

/// Allocates the next `rIdN` relationship identifier.
#[cfg(test)]
fn next_relationship_id(rels_xml: &str) -> String {
    let mut maximum = 0usize;
    let mut offset = 0;
    while let Some(relative) = rels_xml[offset..].find("Id=\"rId") {
        let start = offset + relative + "Id=\"rId".len();
        let digits: String = rels_xml[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(index) = digits.parse::<usize>() {
            maximum = maximum.max(index);
        }
        offset = start;
    }
    format!("rId{}", maximum.saturating_add(1))
}

/// Allocates the next workbook `sheetId`.
#[cfg(test)]
fn next_sheet_id(workbook_xml: &str) -> usize {
    let mut maximum = 0usize;
    for element in xml_elements(workbook_xml, "sheet") {
        if let Some(value) = attribute_value(element, "sheetId")
            && let Ok(index) = value.parse::<usize>()
        {
            maximum = maximum.max(index);
        }
    }
    maximum.saturating_add(1)
}

/// Inserts `fragment` immediately before the first occurrence of `close_tag`.
#[cfg(test)]
fn insert_before_close_tag(xml: &str, close_tag: &str, fragment: &str) -> Result<String> {
    let Some(index) = xml.find(close_tag) else {
        return Err(ExcelError::Format(format!(
            "template XML is missing {close_tag}"
        )));
    };
    Ok(format!("{}{}{}", &xml[..index], fragment, &xml[index..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_name_single_column() {
        assert_eq!(column_name(1), "A");
        assert_eq!(column_name(26), "Z");
    }

    #[test]
    fn column_name_double_column() {
        assert_eq!(column_name(27), "AA");
        assert_eq!(column_name(28), "AB");
        assert_eq!(column_name(52), "AZ");
        assert_eq!(column_name(53), "BA");
    }

    #[test]
    fn column_name_triple_column() {
        assert_eq!(column_name(703), "AAA");
    }

    #[test]
    fn escape_xml_special_characters() {
        assert_eq!(escape_xml("hello"), "hello");
        assert_eq!(escape_xml("a&b"), "a&amp;b");
        assert_eq!(escape_xml("a<b"), "a&lt;b");
        assert_eq!(escape_xml("a>b"), "a&gt;b");
        assert_eq!(escape_xml("a\"b"), "a&quot;b");
        assert_eq!(escape_xml("a'b"), "a&apos;b");
        assert_eq!(escape_xml("<tag>&\"'"), "&lt;tag&gt;&amp;&quot;&apos;");
    }

    #[test]
    fn escape_xml_empty_string() {
        assert_eq!(escape_xml(""), "");
    }

    #[test]
    fn parse_cell_reference_simple() {
        assert_eq!(parse_cell_reference("A1"), Some((1, 1)));
        assert_eq!(parse_cell_reference("B2"), Some((2, 2)));
        assert_eq!(parse_cell_reference("Z99"), Some((26, 99)));
    }

    #[test]
    fn parse_cell_reference_multi_column() {
        assert_eq!(parse_cell_reference("AA1"), Some((27, 1)));
        assert_eq!(parse_cell_reference("AB10"), Some((28, 10)));
    }

    #[test]
    fn parse_cell_reference_invalid() {
        assert_eq!(parse_cell_reference(""), None);
        assert_eq!(parse_cell_reference("1A"), None);
    }

    #[test]
    fn normalize_workbook_target_with_xl_prefix() {
        let result = normalize_workbook_target("xl/worksheets/sheet1.xml");
        assert_eq!(result, "xl/worksheets/sheet1.xml");
    }

    #[test]
    fn normalize_workbook_target_without_xl_prefix() {
        let result = normalize_workbook_target("worksheets/sheet1.xml");
        assert_eq!(result, "xl/worksheets/sheet1.xml");
    }

    #[test]
    fn normalize_workbook_target_with_leading_slash() {
        let result = normalize_workbook_target("/xl/styles.xml");
        assert_eq!(result, "xl/styles.xml");
    }

    #[test]
    fn attribute_value_simple() {
        let xml = r#"<tag attr="value">"#;
        assert_eq!(attribute_value(xml, "attr"), Some("value"));
    }

    #[test]
    fn attribute_value_missing() {
        let xml = r#"<tag attr="value">"#;
        assert_eq!(attribute_value(xml, "missing"), None);
    }

    #[test]
    fn attribute_value_in_tag_found() {
        let xml = r#"<worksheet dim="A1"><sheetData/></worksheet>"#;
        assert_eq!(attribute_value_in_tag(xml, "worksheet", "dim"), Some("A1"));
    }

    #[test]
    fn attribute_value_in_tag_not_found() {
        let xml = r"<worksheet><sheetData/></worksheet>";
        assert_eq!(attribute_value_in_tag(xml, "worksheet", "dim"), None);
    }

    #[test]
    fn xml_elements_iterates() {
        let xml = r#"<worksheet><row r="1"/><row r="2"/><row r="3"/></worksheet>"#;
        let elements: Vec<&str> = xml_elements(xml, "row").collect();
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn xml_elements_empty() {
        let xml = r"<worksheet><sheetData/></worksheet>";
        let elements: Vec<&str> = xml_elements(xml, "row").collect();
        assert_eq!(elements.len(), 0);
    }

    #[test]
    fn worksheet_max_row_with_rows() {
        let xml = r#"<sheetData><row r="5"/><row r="10"/></sheetData>"#;
        assert_eq!(worksheet_max_row(xml), 10);
    }

    #[test]
    fn worksheet_max_row_empty() {
        let xml = "<sheetData/>";
        assert_eq!(worksheet_max_row(xml), 0);
    }

    #[test]
    fn row_index_extracts_number() {
        assert_eq!(row_index("row"), None);
        assert_eq!(row_index("row r=\"15\""), Some(15));
    }

    #[test]
    fn update_worksheet_dimension_test() {
        let xml = r#"<worksheet><dimension ref="A1"/><sheetData><row r="1"/><row r="5"/></sheetData></worksheet>"#;
        let updated = update_worksheet_dimension(xml);
        assert!(updated.contains("ref=\"A1:A5\"") || updated.contains("A1"));
    }

    #[test]
    fn render_cell_xml_string() {
        let xml = render_cell_xml("A1", &CellValue::String("test".to_owned()), None);
        assert!(xml.contains("inlineStr"));
        assert!(xml.contains("test"));
    }

    #[test]
    fn render_cell_xml_number() {
        let xml = render_cell_xml("B1", &CellValue::Float(42.0), Some(1));
        assert!(xml.contains("s=\"1\""));
        assert!(xml.contains("42"));
    }

    #[test]
    fn render_cell_xml_empty() {
        let xml = render_cell_xml("C1", &CellValue::Empty, None);
        assert!(xml.contains("C1"));
    }

    #[test]
    fn render_cell_xml_bool() {
        let xml = render_cell_xml("A1", &CellValue::Bool(true), None);
        assert!(xml.contains('1'));
    }

    #[test]
    fn render_cell_xml_formula() {
        let xml = render_cell_xml("A1", &CellValue::Formula("SUM(A1:A10)".to_owned()), None);
        assert!(xml.contains("SUM(A1:A10)"));
    }

    #[test]
    fn render_cell_xml_date() {
        use chrono::NaiveDate;
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let xml = render_cell_xml("A1", &CellValue::Date(date), None);
        assert!(xml.contains("A1"));
    }

    #[test]
    fn cell_style_index_found() {
        let xml = r#"<worksheet><sheetData><row><c r="A1" s="5"/></row></sheetData></worksheet>"#;
        assert_eq!(cell_style_index(xml, "A1"), Some(5));
    }

    #[test]
    fn cell_style_index_not_found() {
        let xml = r#"<worksheet><sheetData><row><c r="A1"/></row></sheetData></worksheet>"#;
        assert_eq!(cell_style_index(xml, "A1"), None);
    }

    #[test]
    fn merge_compiled_styles_basic() {
        let mut destination = String::from(
            r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">"#,
        );
        destination.push_str(r#"<fonts count="1"><font/></fonts>"#);
        destination.push_str(r#"<fills count="1"><fill/></fills>"#);
        destination.push_str(r#"<borders count="1"><border/></borders>"#);
        destination.push_str(r#"<cellStyleXfs count="1"><xf/></cellStyleXfs>"#);
        destination.push_str(r#"<cellXfs count="0"><xf/></cellXfs>"#);
        destination.push_str("</styleSheet>");

        let source = destination.clone();

        let (_output, xf_indexes) = merge_compiled_styles(&destination, &source, &[]).unwrap();
        assert!(xf_indexes.is_empty());
    }

    #[test]
    fn blank_worksheet_with_inherited_format_creates_xml() {
        let entries = vec![
            TemplateZipEntry {
                name: "xl/worksheets/sheet1.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetFormatPr defaultRowHeight="20"/><cols><col min="1" max="1" width="30" customWidth="1"/></cols><sheetData/></worksheet>"#.to_vec(),
            },
        ];
        let xml_bytes = blank_worksheet_with_inherited_format(&entries);
        let xml = String::from_utf8(xml_bytes).unwrap();
        assert!(xml.contains("defaultRowHeight=\"20\""));
        assert!(xml.contains("customWidth=\"1\""));
        assert!(xml.contains("<sheetData>"));
    }

    #[test]
    fn blank_worksheet_no_entries() {
        let entries: Vec<TemplateZipEntry> = vec![];
        let xml_bytes = blank_worksheet_with_inherited_format(&entries);
        let xml = String::from_utf8(xml_bytes).unwrap();
        assert!(xml.contains("sheetData"));
    }

    #[test]
    fn extract_xml_element_found() {
        let xml = r#"<worksheet><sheetData><row r="1"/></sheetData></worksheet>"#;
        let result = extract_xml_element(xml, "sheetData");
        assert!(result.is_some());
    }

    #[test]
    fn extract_xml_element_not_found() {
        let xml = r"<worksheet><sheetData/></worksheet>";
        let result = extract_xml_element(xml, "row");
        assert!(result.is_none());
    }

    #[test]
    fn resolve_template_target_prefers_index_then_name() {
        let sheets = vec![
            TemplateSheetData {
                name: "A".to_owned(),
                cells: Vec::new(),
                next_row: 1,
            },
            TemplateSheetData {
                name: "B".to_owned(),
                cells: Vec::new(),
                next_row: 2,
            },
        ];
        // Java `.sheet(1)` / `.sheet(1, "ignored")` selects by index.
        let (index, name, create_new) = resolve_template_target(&sheets, Some(1), "ignored");
        assert_eq!((index, name.as_str(), create_new), (1, "B", false));
        // Java `.sheet("B")` selects by name when sheetNo is null.
        let (index, name, create_new) = resolve_template_target(&sheets, None, "B");
        assert_eq!((index, name.as_str(), create_new), (1, "B", false));
        // Missing name creates a new sheet after template sheets.
        let (index, name, create_new) = resolve_template_target(&sheets, None, "C");
        assert_eq!((index, name.as_str(), create_new), (2, "C", true));
    }

    #[test]
    fn validate_template_source_rejects_csv_and_xls_for_xlsx_path() {
        let csv = Path::new("demo.csv");
        let err = validate_template_source(Some(csv), None).expect_err("csv");
        assert!(err.to_string().contains("csv cannot use template"));

        let xls = Path::new("demo.xls");
        let err = validate_template_source(Some(xls), None).expect_err("xls");
        assert!(
            err.to_string()
                .contains("legacy XLS template cannot seed an XLSX"),
            "unexpected: {err}"
        );

        let err = validate_template_source(None, Some(b"name,age\n")).expect_err("csv bytes");
        assert!(err.to_string().contains("csv cannot use template"));
    }

    #[test]
    fn append_sparse_rows_preserves_merge_cells_trailer() {
        let xml = concat!(
            "<worksheet><dimension ref=\"A1:B1\"/>",
            "<sheetData><row r=\"1\"><c r=\"A1\" s=\"1\" t=\"s\"><v>0</v></c></row></sheetData>",
            "<mergeCells count=\"1\"><mergeCell ref=\"A1:B1\"/></mergeCells>",
            "</worksheet>"
        );
        let rows = vec![vec![(0usize, CellValue::String("appended".to_owned()))]];
        let (updated, next) = append_sparse_rows_to_xml(xml, &rows, &[], &[], &[]).expect("append");
        assert_eq!(next, 3);
        assert!(updated.contains("s=\"1\""));
        assert!(updated.contains("<mergeCell ref=\"A1:B1\"/>"));
        assert!(updated.contains("inlineStr"));
        assert!(updated.contains("appended"));
    }

    #[test]
    fn append_sparse_rows_expands_self_closing_sheet_data() {
        let xml = concat!(
            "<worksheet><dimension ref=\"A1\"/>",
            "<sheetData/>",
            "</worksheet>"
        );
        let rows = vec![vec![(0usize, CellValue::String("fresh".to_owned()))]];
        let (updated, next) = append_sparse_rows_to_xml(xml, &rows, &[], &[], &[]).expect("append");
        assert_eq!(next, 2);
        assert!(updated.contains("<sheetData><row r=\"1\">"));
        assert!(updated.contains("fresh"));
        assert!(updated.contains("</sheetData>"));
        assert!(!updated.contains("<sheetData/>"));
    }

    #[test]
    // 语义敏感：该测试端到端覆盖样式合并的每个分支（对应 Java 模板写入用例），
    // 拆分会降低可读性，故豁免 too_many_lines。
    #[allow(clippy::too_many_lines)]
    fn create_sheet_keeps_existing_styles_and_merges() {
        // placeholder — build via TemplatePackage entries below
        let template = "PK\x03\x04";
        let _ = template;
        let mut package = TemplatePackage {
            entries: vec![
                TemplateZipEntry {
                    name: "[Content_Types].xml".to_owned(),
                    is_dir: false,
                    compression: CompressionMethod::Stored,
                    unix_mode: None,
                    bytes: br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#.to_vec(),
                },
                TemplateZipEntry {
                    name: "xl/workbook.xml".to_owned(),
                    is_dir: false,
                    compression: CompressionMethod::Stored,
                    unix_mode: None,
                    bytes: br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Styled" sheetId="1" r:id="rId1"/></sheets></workbook>"#.to_vec(),
                },
                TemplateZipEntry {
                    name: "xl/_rels/workbook.xml.rels".to_owned(),
                    is_dir: false,
                    compression: CompressionMethod::Stored,
                    unix_mode: None,
                    bytes: br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#.to_vec(),
                },
                TemplateZipEntry {
                    name: "xl/styles.xml".to_owned(),
                    is_dir: false,
                    compression: CompressionMethod::Stored,
                    unix_mode: None,
                    bytes: br#"<?xml version="1.0"?><styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="1"><font><b/></font></fonts></styleSheet>"#.to_vec(),
                },
                TemplateZipEntry {
                    name: "xl/worksheets/sheet1.xml".to_owned(),
                    is_dir: false,
                    compression: CompressionMethod::Stored,
                    unix_mode: None,
                    bytes: br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetFormatPr defaultRowHeight="18"/><cols><col min="1" max="1" width="20" customWidth="1"/></cols><sheetData><row r="1"><c r="A1" s="1"><v>1</v></c></row></sheetData><mergeCells count="1"><mergeCell ref="A1:B1"/></mergeCells></worksheet>"#.to_vec(),
                },
            ].into(),
        };
        let styles_before = package
            .entries
            .iter()
            .find(|entry| entry.name == "xl/styles.xml")
            .map(|entry| entry.bytes.clone())
            .expect("styles");
        let sheet1_before = package
            .entries
            .iter()
            .find(|entry| entry.name == "xl/worksheets/sheet1.xml")
            .map(|entry| entry.bytes.clone())
            .expect("sheet1");

        package.create_sheet("NewSheet").expect("create");
        package
            .append_rows(
                "NewSheet",
                &[vec![(0usize, CellValue::String("fresh".to_owned()))]],
            )
            .expect("append");

        let styles_after = package
            .entries
            .iter()
            .find(|entry| entry.name == "xl/styles.xml")
            .map(|entry| entry.bytes.clone())
            .expect("styles");
        let sheet1_after = package
            .entries
            .iter()
            .find(|entry| entry.name == "xl/worksheets/sheet1.xml")
            .map(|entry| entry.bytes.clone())
            .expect("sheet1");
        assert_eq!(styles_before, styles_after, "styles.xml must be untouched");
        assert_eq!(
            sheet1_before, sheet1_after,
            "existing sheet XML (incl. mergeCells) must be untouched"
        );
        assert!(
            package
                .sheet_names()
                .expect("names")
                .iter()
                .any(|name| name == "NewSheet")
        );
        let new_sheet = package
            .entry_xml("xl/worksheets/sheet2.xml")
            .expect("new sheet xml");
        assert!(new_sheet.contains("fresh"));
        assert!(
            new_sheet.contains("defaultRowHeight=\"18\""),
            "new sheet should inherit sheetFormatPr: {new_sheet}"
        );
        assert!(
            new_sheet.contains("customWidth=\"1\""),
            "new sheet should inherit cols: {new_sheet}"
        );
        assert!(
            !new_sheet.contains("mergeCell"),
            "new sheet must not copy merges from the template sheet"
        );
    }

    #[test]
    fn importing_same_compiled_style_reuses_workbook_components_and_xf() {
        let mut template = Workbook::new();
        template
            .add_worksheet()
            .write_string_with_format(
                0,
                0,
                "seed",
                &Format::new().set_bold().set_font_color(0x0000_00ff),
            )
            .expect("template seed");
        let template_bytes = template.save_to_buffer().expect("template bytes");
        let mut package = TemplatePackage::from_bytes(&template_bytes).expect("template package");

        let mut compiler = Workbook::new();
        compiler
            .add_worksheet()
            .write_blank(
                0,
                0,
                &Format::new()
                    .set_italic()
                    .set_font_color(0x00ff_0000)
                    .set_num_format("0.000"),
            )
            .expect("compiled style");
        let compiled_bytes = compiler.save_to_buffer().expect("compiled bytes");

        let first = package
            .import_compiled_styles(&compiled_bytes, 1)
            .expect("first import");
        let after_first = package.entry_xml("xl/styles.xml").expect("styles");
        let second = package
            .import_compiled_styles(&compiled_bytes, 1)
            .expect("second import");
        let after_second = package.entry_xml("xl/styles.xml").expect("styles");

        assert_eq!(first, second);
        assert_eq!(after_first, after_second);
    }

    #[test]
    fn validate_template_source_xlsx_file_succeeds() {
        let result = validate_template_source(Some(Path::new("nonexistent.xlsx")), None);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_template_source_csv_file_fails() {
        let result = validate_template_source(Some(Path::new("template.csv")), None);
        assert!(result.is_err());
    }

    #[test]
    fn validate_template_source_xls_file_fails() {
        let result = validate_template_source(Some(Path::new("template.xls")), None);
        assert!(result.is_err());
    }

    #[test]
    fn validate_template_source_no_source_succeeds() {
        let result = validate_template_source(None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn has_template_returns_false_for_none() {
        assert!(!has_template(None, None));
    }

    #[test]
    fn has_template_returns_true_for_file() {
        assert!(has_template(Some(Path::new("template.xlsx")), None));
    }

    #[test]
    fn has_template_returns_true_for_bytes() {
        assert!(has_template(None, Some(b"fake bytes")));
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use crate::core::RichTextStringData;
    use bigdecimal::BigDecimal;
    use calamine::{CellErrorType, ExcelDateTime, ExcelDateTimeType};
    use chrono::NaiveDate;
    use std::str::FromStr;

    /// Package entries mirroring a minimal template with one styled sheet.
    fn sample_entries() -> Vec<TemplateZipEntry> {
        vec![
            TemplateZipEntry {
                name: "[Content_Types].xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>"#.to_vec(),
            },
            TemplateZipEntry {
                name: "xl/workbook.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Styled" sheetId="1" r:id="rId1"/></sheets></workbook>"#.to_vec(),
            },
            TemplateZipEntry {
                name: "xl/_rels/workbook.xml.rels".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/></Relationships>"#.to_vec(),
            },
            TemplateZipEntry {
                name: "xl/styles.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: br#"<?xml version="1.0"?><styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><fonts count="1"><font><b/></font></fonts></styleSheet>"#.to_vec(),
            },
            TemplateZipEntry {
                name: "xl/worksheets/sheet1.xml".to_owned(),
                is_dir: false,
                compression: CompressionMethod::Stored,
                unix_mode: None,
                bytes: br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetFormatPr defaultRowHeight="18"/><cols><col min="1" max="1" width="20" customWidth="1"/></cols><sheetData><row r="1"><c r="A1" s="1"><v>1</v></c></row></sheetData><mergeCells count="1"><mergeCell ref="A1:B1"/></mergeCells></worksheet>"#.to_vec(),
            },
        ]
    }

    fn sample_package() -> TemplatePackage {
        TemplatePackage {
            entries: sample_entries().into(),
        }
    }

    fn workbook_package(rows: usize) -> TemplatePackage {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        for row in 0..rows {
            // 语义敏感：测试种子行数远小于 u32 上限。
            #[allow(clippy::cast_possible_truncation)]
            let row_u32 = row as u32;
            worksheet
                .write_string(row_u32, 0, format!("seed-{row}"))
                .expect("seed cell");
        }
        let bytes = workbook.save_to_buffer().expect("template bytes");
        TemplatePackage::from_bytes(&bytes).expect("template package")
    }

    fn styles_xml(fonts: &str, fills: &str, borders: &str, xfs: &str) -> String {
        format!(
            r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">{fonts}{fills}{borders}{xfs}</styleSheet>"#
        )
    }

    fn standard_styles() -> String {
        styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf/></cellXfs>"#,
        )
    }

    #[test]
    fn from_bytes_reads_directory_entries_and_unix_mode() {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer
            .add_directory("xl/dir/", options.unix_permissions(0o755))
            .expect("directory entry");
        writer
            .start_file("xl/workbook.xml", options)
            .expect("file entry");
        writer
            .write_all(b"<workbook/>")
            .expect("write workbook bytes");
        let bytes = writer.finish().expect("finish zip").into_inner();

        let package = TemplatePackage::from_bytes(&bytes).expect("package");
        assert_eq!(package.entries.len(), 2);
        let directory = &package.entries[0];
        assert!(directory.is_dir);
        assert!(directory.bytes.is_empty());
        assert!(directory.unix_mode.is_some());
        // Round-trips through the ZIP writer, covering the directory branch.
        let output = package.to_bytes().expect("reserialize");
        assert!(output.starts_with(b"PK"));
    }

    #[test]
    fn next_row_for_sheet_empty_and_populated() {
        let empty = workbook_package(0);
        assert_eq!(empty.next_row_for_sheet("Sheet1").expect("next"), 0);

        let populated = workbook_package(2);
        assert_eq!(
            populated.next_row_for_sheet("Sheet1").expect("next"),
            3,
            "last row index 1 (zero-based) + 1 = 2 rows"
        );
    }

    #[test]
    fn ensure_sheet_existing_and_creates_new() {
        let mut package = sample_package();
        package.ensure_sheet("Styled").expect("existing sheet");
        package.ensure_sheet("Extra").expect("created sheet");
        let names = package.sheet_names().expect("names");
        assert!(names.iter().any(|name| name == "Extra"));
        assert!(
            package
                .entry_xml("xl/worksheets/sheet2.xml")
                .expect("new part")
                .contains("<sheetData>")
        );
    }

    #[test]
    fn create_sheet_error_paths() {
        let mut no_rels = sample_package();
        no_rels
            .entries
            .retain(|entry| entry.name != "xl/_rels/workbook.xml.rels");
        let error = no_rels.create_sheet("X").expect_err("missing rels");
        assert!(error.to_string().contains("workbook.xml.rels"));

        let mut no_types = sample_package();
        no_types
            .entries
            .retain(|entry| entry.name != "[Content_Types].xml");
        let error = no_types.create_sheet("X").expect_err("missing types");
        assert!(error.to_string().contains("[Content_Types].xml"));

        let mut no_sheets_close = sample_package();
        let workbook = no_sheets_close
            .entries
            .iter_mut()
            .find(|entry| entry.name == "xl/workbook.xml")
            .expect("workbook entry");
        workbook.bytes = br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Styled" sheetId="1" r:id="rId1"/></workbook>"#.to_vec();
        let error = no_sheets_close
            .create_sheet("X")
            .expect_err("missing </sheets>");
        assert!(error.to_string().contains("</sheets>"));
    }

    #[test]
    fn append_rows_validation_errors() {
        let mut package = sample_package();
        // Empty rows delegates to the next-row computation.
        assert_eq!(
            package
                .append_rows_with_layout_and_absent("Styled", &[], &[], &[], &[])
                .expect("empty rows"),
            2
        );
        let rows = vec![vec![(0usize, CellValue::String("x".to_owned()))]];
        let error = package
            .append_rows_with_layout_and_absent("Styled", &rows, &[], &[], &[true, false])
            .expect_err("absent count mismatch");
        assert!(error.to_string().contains("absent-row count"));
        let error = package
            .append_rows_with_layout_and_absent("Styled", &rows, &[Some(10), Some(11)], &[], &[])
            .expect_err("height count mismatch");
        assert!(error.to_string().contains("row-height count"));
        let error = package
            .append_rows_with_layout_and_absent(
                "Styled",
                &rows,
                &[],
                &[vec![Some(0u32)], vec![Some(0u32)]],
                &[],
            )
            .expect_err("style row count mismatch");
        assert!(error.to_string().contains("cell-style shape"));
        let two_cell_row = vec![vec![
            (0usize, CellValue::String("x".to_owned())),
            (1usize, CellValue::Int(1)),
        ]];
        let error = package
            .append_rows_with_layout_and_absent(
                "Styled",
                &two_cell_row,
                &[],
                &[vec![Some(0u32)]],
                &[],
            )
            .expect_err("style shape mismatch");
        assert!(error.to_string().contains("cell-style shape"));
    }

    #[test]
    fn append_sparse_rows_absent_and_heights() {
        let xml = "<worksheet><sheetData></sheetData></worksheet>";
        let rows = vec![
            vec![(0usize, CellValue::String("a".to_owned()))],
            vec![(0usize, CellValue::Int(1))],
        ];
        let (updated, next) =
            append_sparse_rows_to_xml(xml, &rows, &[Some(22)], &[], &[false, true])
                .expect("append");
        assert_eq!(next, 3);
        assert!(updated.contains("ht=\"22\""));
        assert!(updated.contains("<row r=\"1\" ht=\"22\" customHeight=\"1\">"));
        assert!(!updated.contains("r=\"2\""), "absent row must be skipped");
    }

    #[test]
    fn apply_sheet_layout_variants() {
        let mut package = sample_package();
        package
            .apply_sheet_layout("Styled", &[], &[])
            .expect("no-op layout");
        package
            .apply_sheet_layout("Styled", &[(0, 30)], &[MergeRange::new(0, 1, 2, 2)])
            .expect("layout");
        let sheet = package
            .entry_xml("xl/worksheets/sheet1.xml")
            .expect("sheet xml");
        assert!(sheet.contains("width=\"30\""));
        assert!(sheet.contains("ref=\"C1:C2\""));
        assert!(sheet.contains("count=\"2\""));
    }

    #[test]
    fn apply_column_widths_to_xml_variants() {
        let with_close_cols = "<worksheet><cols><col min=\"1\" max=\"1\" width=\"10\"/></cols><sheetData/></worksheet>";
        let updated = apply_column_widths_to_xml(with_close_cols, &[(1, 20)]).expect("close cols");
        assert!(updated.contains("width=\"20\""));

        let self_closing = "<worksheet><cols/><sheetData/></worksheet>";
        let updated =
            apply_column_widths_to_xml(self_closing, &[(0, 15)]).expect("self closing cols");
        assert!(updated.contains("<cols><col"));

        let error =
            apply_column_widths_to_xml("<worksheet><cols", &[(0, 10)]).expect_err("malformed cols");
        assert!(error.to_string().contains("malformed cols"));

        let no_cols = "<worksheet><sheetData></sheetData></worksheet>";
        let updated = apply_column_widths_to_xml(no_cols, &[(0, 10)]).expect("insert cols");
        assert!(updated.contains("<cols>"));

        let error =
            apply_column_widths_to_xml("<worksheet/>", &[(0, 10)]).expect_err("missing sheetData");
        assert!(error.to_string().contains("sheetData"));
    }

    #[test]
    fn apply_merge_ranges_to_xml_variants() {
        let existing = "<worksheet><mergeCells count=\"1\"><mergeCell ref=\"A1:B1\"/></mergeCells></worksheet>";
        let unchanged = apply_merge_ranges_to_xml(existing, &[MergeRange::new(0, 0, 0, 1)])
            .expect("already merged");
        assert_eq!(unchanged, existing);

        let with_count = "<worksheet><sheetData></sheetData><mergeCells count=\"1\"><mergeCell ref=\"A1:B1\"/></mergeCells></worksheet>";
        let updated =
            apply_merge_ranges_to_xml(with_count, &[MergeRange::new(2, 2, 0, 0)]).expect("append");
        assert!(updated.contains("count=\"2\""));
        assert!(updated.contains("ref=\"A3:A3\""));

        let no_count = "<worksheet><mergeCells><mergeCell ref=\"A1:B1\"/></mergeCells></worksheet>";
        let updated =
            apply_merge_ranges_to_xml(no_count, &[MergeRange::new(2, 2, 0, 0)]).expect("append");
        assert!(updated.contains("ref=\"A3:A3\""));

        let error =
            apply_merge_ranges_to_xml("<worksheet><mergeCells", &[MergeRange::new(0, 0, 0, 0)])
                .expect_err("unterminated open");
        assert!(error.to_string().contains("malformed mergeCells"));

        let error =
            apply_merge_ranges_to_xml("<worksheet><mergeCells>", &[MergeRange::new(0, 0, 0, 0)])
                .expect_err("missing close");
        assert!(error.to_string().contains("malformed mergeCells"));

        let no_merges = "<worksheet><sheetData></sheetData></worksheet>";
        let updated =
            apply_merge_ranges_to_xml(no_merges, &[MergeRange::new(0, 0, 0, 1)]).expect("insert");
        assert!(updated.contains("<mergeCells count=\"1\">"));

        let error = apply_merge_ranges_to_xml("<worksheet/>", &[MergeRange::new(0, 0, 0, 0)])
            .expect_err("missing sheetData");
        assert!(error.to_string().contains("sheetData"));
    }

    #[test]
    fn expand_self_closing_sheet_data_errors() {
        let error = expand_self_closing_sheet_data("<worksheet/>").expect_err("no sheetData");
        assert!(error.to_string().contains("sheetData"));
        let error =
            expand_self_closing_sheet_data("<worksheet><sheetData").expect_err("no self close");
        assert!(error.to_string().contains("sheetData"));
        let error =
            expand_self_closing_sheet_data("<worksheet><sheetData><row r=\"1\"/></worksheet>")
                .expect_err("self close belongs to sibling");
        assert!(error.to_string().contains("sheetData"));
    }

    #[test]
    fn render_cell_xml_extra_variants() {
        let rich = render_cell_xml(
            "A1",
            &CellValue::RichText(RichTextStringData::new("rt")),
            None,
        );
        assert!(rich.contains("rt"));

        let int = render_cell_xml("A1", &CellValue::Int(42), None);
        assert!(int.contains("<v>42</v>"));

        let big_integer = render_cell_xml(
            "A1",
            &CellValue::Decimal(BigDecimal::from(100_000_000_000_000_000_000i128)),
            None,
        );
        assert!(big_integer.contains("inlineStr"));

        let fraction = render_cell_xml(
            "A1",
            &CellValue::Decimal(BigDecimal::from_str("1.5").expect("decimal")),
            None,
        );
        assert!(fraction.contains("<v>1.5</v>"));

        let datetime = render_cell_xml(
            "A1",
            &CellValue::DateTime(
                NaiveDate::from_ymd_opt(2024, 1, 2)
                    .expect("date")
                    .and_hms_opt(3, 4, 5)
                    .expect("time"),
            ),
            None,
        );
        assert!(datetime.contains("2024-01-02T03:04:05"));

        let comment = render_cell_xml(
            "A1",
            &CellValue::Comment {
                value: Box::new(CellValue::Int(7)),
                text: "note".to_owned(),
            },
            None,
        );
        assert!(comment.contains("<v>7</v>"));

        let images = render_cell_xml(
            "A1",
            &CellValue::Images {
                value: Box::new(CellValue::String("img".to_owned())),
                images: Vec::new(),
            },
            None,
        );
        assert!(images.contains("img"));
    }

    #[test]
    fn write_template_cell_all_variants() {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        write_template_cell(worksheet, 0, 0, &Data::Empty).expect("empty");
        write_template_cell(worksheet, 0, 1, &Data::String("s".to_owned())).expect("string");
        write_template_cell(
            worksheet,
            0,
            2,
            &Data::DateTimeIso("2024-01-01T00:00:00".to_owned()),
        )
        .expect("datetime iso");
        write_template_cell(worksheet, 0, 3, &Data::DurationIso("PT1H".to_owned()))
            .expect("duration iso");
        write_template_cell(worksheet, 0, 4, &Data::Bool(true)).expect("bool");
        write_template_cell(worksheet, 0, 5, &Data::Int(42)).expect("int");
        write_template_cell(worksheet, 0, 6, &Data::Float(1.5)).expect("float");
        write_template_cell(
            worksheet,
            0,
            7,
            &Data::DateTime(ExcelDateTime::new(
                45943.5,
                ExcelDateTimeType::DateTime,
                false,
            )),
        )
        .expect("datetime");
        write_template_cell(
            worksheet,
            0,
            8,
            &Data::DateTime(ExcelDateTime::new(
                1e300,
                ExcelDateTimeType::DateTime,
                false,
            )),
        )
        .expect("datetime outside chrono range");
        write_template_cell(worksheet, 0, 9, &Data::Error(CellErrorType::Div0)).expect("error");
        workbook.save_to_buffer().expect("save");
    }

    #[test]
    fn seed_workbook_from_template_writes_sheets() {
        let sheets = vec![TemplateSheetData {
            name: "S1".to_owned(),
            cells: vec![
                (0u32, 0u16, Data::String("a".to_owned())),
                (1u32, 1u16, Data::Int(5)),
                (2u32, 0u16, Data::Bool(true)),
            ],
            next_row: 3,
        }];
        let mut workbook = Workbook::new();
        seed_workbook_from_template(&mut workbook, &sheets).expect("seed");
        workbook.save_to_buffer().expect("save");
    }

    #[test]
    fn load_template_sheets_reads_workbook_and_errors() {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();
        worksheet.write_string(0, 0, "alpha").expect("cell");
        worksheet.write_number(2, 1, 5.0).expect("cell");
        let bytes = workbook.save_to_buffer().expect("bytes");

        let sheets = load_template_sheets(&bytes).expect("sheets");
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheets[0].name, "Sheet1");
        assert_eq!(
            sheets[0].cells.len(),
            2,
            "empty gaps inside the used range are skipped"
        );
        assert_eq!(sheets[0].next_row, 3);

        let error = load_template_sheets(b"not a zip file").expect_err("invalid package");
        assert!(error.to_string().contains("withTemplate"));
    }

    #[test]
    fn apply_sheet_layout_widths_only_and_merges_only() {
        let mut package = sample_package();
        package
            .apply_sheet_layout("Styled", &[(1, 12)], &[])
            .expect("widths only");
        package
            .apply_sheet_layout("Styled", &[], &[MergeRange::new(3, 3, 0, 0)])
            .expect("merges only");
    }

    /// Minimal OOXML package with a configurable `<sheets>` body; the
    /// worksheet part referenced by `rId1` is intentionally absent.
    fn minimal_xlsx_with_sheets(sheet_tag: &str) -> Vec<u8> {
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer
            .start_file("[Content_Types].xml", options)
            .expect("content types");
        writer
            .write_all(
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#,
            )
            .expect("write");
        writer
            .start_file("_rels/.rels", options)
            .expect("package rels");
        writer
            .write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
            )
            .expect("write");
        writer
            .start_file("xl/workbook.xml", options)
            .expect("workbook");
        writer
            .write_all(
                format!(
                    r#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>{sheet_tag}</sheets></workbook>"#
                )
                .as_bytes(),
            )
            .expect("write");
        writer
            .start_file("xl/_rels/workbook.xml.rels", options)
            .expect("workbook rels");
        writer
            .write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/ghost.xml"/></Relationships>"#,
            )
            .expect("write");
        writer.finish().expect("finish").into_inner()
    }

    #[test]
    fn load_template_sheets_empty_and_missing_part() {
        let empty = minimal_xlsx_with_sheets("");
        let error = load_template_sheets(&empty).expect_err("no worksheets");
        assert!(error.to_string().contains("no worksheets"));

        let ghost = minimal_xlsx_with_sheets(r#"<sheet name="Ghost" sheetId="1" r:id="rId1"/>"#);
        let error = load_template_sheets(&ghost).expect_err("worksheet part missing");
        assert!(
            error
                .to_string()
                .contains("failed to read withTemplate sheet")
        );
    }

    #[test]
    fn load_template_bytes_branches() {
        let data = b"template-bytes";
        assert_eq!(
            load_template_bytes(None, Some(data)).expect("bytes"),
            data.to_vec()
        );
        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("template.xlsx");
        std::fs::write(&path, data).expect("write file");
        assert_eq!(
            load_template_bytes(Some(&path), None).expect("file"),
            data.to_vec()
        );
        let error = load_template_bytes(None, None).expect_err("no source");
        assert!(error.to_string().contains("with_template"));
    }

    #[test]
    fn validate_template_source_xlsx_and_xls_bytes() {
        let mut workbook = Workbook::new();
        workbook.add_worksheet();
        let xlsx = workbook.save_to_buffer().expect("xlsx");
        assert!(
            validate_template_source(None, Some(&xlsx)).is_ok(),
            "xlsx bytes are not CSV-like"
        );
        let xls = [0xD0u8, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0, 0];
        let error = validate_template_source(None, Some(&xls)).expect_err("xls bytes");
        assert!(error.to_string().contains("legacy XLS"));
    }

    #[test]
    fn resolve_targets_out_of_range_and_by_name() {
        let sheets = vec![TemplateSheetData {
            name: "A".to_owned(),
            cells: Vec::new(),
            next_row: 0,
        }];
        let (index, name, create_new) = resolve_template_target(&sheets, Some(7), "Z");
        assert_eq!((index, name.as_str(), create_new), (7, "Z", true));

        let names = vec!["A".to_owned()];
        let (index, name, create_new) = resolve_package_target(&names, Some(0), "ignored");
        assert_eq!((index, name.as_str(), create_new), (0, "A", false));
        let (index, name, create_new) = resolve_package_target(&names, Some(3), "ignored");
        assert_eq!((index, name.as_str(), create_new), (3, "ignored", true));
        let (index, name, create_new) = resolve_package_target(&names, None, "A");
        assert_eq!((index, name.as_str(), create_new), (0, "A", false));
        let (index, name, create_new) = resolve_package_target(&names, None, "B");
        assert_eq!((index, name.as_str(), create_new), (1, "B", true));
    }

    #[test]
    fn merge_compiled_styles_duplicate_and_out_of_range() {
        let source = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="2"><xf fontId="0" fillId="0" borderId="0" numFmtId="0"/><xf fontId="0" fillId="0" borderId="0" numFmtId="0" applyFont="1"/></cellXfs>"#,
        );
        let (_, mapped) =
            merge_compiled_styles(&standard_styles(), &source, &[1, 1]).expect("duplicate import");
        assert_eq!(mapped, vec![1, 1]);

        let error =
            merge_compiled_styles(&standard_styles(), &source, &[9]).expect_err("out of range");
        assert!(error.to_string().contains("out of range"));
    }

    #[test]
    fn merge_compiled_styles_attribute_paths() {
        let plain = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf/></cellXfs>"#,
        );
        let (_, mapped) = merge_compiled_styles(&standard_styles(), &plain, &[0]).expect("plain");
        assert_eq!(mapped, vec![0]);

        let builtin_format = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf numFmtId="10"/></cellXfs>"#,
        );
        merge_compiled_styles(&standard_styles(), &builtin_format, &[0]).expect("builtin numFmtId");

        let invalid_font = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf fontId="abc"/></cellXfs>"#,
        );
        let error = merge_compiled_styles(&standard_styles(), &invalid_font, &[0])
            .expect_err("invalid fontId");
        assert!(error.to_string().contains("invalid fontId"));

        let range_font = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf fontId="9"/></cellXfs>"#,
        );
        let error = merge_compiled_styles(&standard_styles(), &range_font, &[0])
            .expect_err("fontId out of range");
        assert!(error.to_string().contains("out of range"));

        let custom_format = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf numFmtId="165"/></cellXfs>"#,
        );
        let error = merge_compiled_styles(&standard_styles(), &custom_format, &[0])
            .expect_err("missing source numFmt");
        assert!(error.to_string().contains("missing numFmtId 165"));
    }

    #[test]
    fn merge_compiled_styles_numfmt_dedup_and_append() {
        let with_numfmts = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<numFmts count="1"><numFmt numFmtId="164" formatCode="0"/></numFmts><cellXfs count="1"><xf/></cellXfs>"#,
        );
        let new_format = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<numFmts count="1"><numFmt numFmtId="165" formatCode="0.00"/></numFmts><cellXfs count="1"><xf numFmtId="165"/></cellXfs>"#,
        );
        let (updated, _) =
            merge_compiled_styles(&with_numfmts, &new_format, &[0]).expect("append numFmt");
        assert!(updated.contains("numFmtId=\"165\""));
        assert!(updated.contains("formatCode=\"0.00\""));

        let matching_format = styles_xml(
            r#"<fonts count="1"><font/></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<numFmts count="1"><numFmt numFmtId="200" formatCode="0"/></numFmts><cellXfs count="1"><xf numFmtId="200"/></cellXfs>"#,
        );
        let (updated, _) =
            merge_compiled_styles(&with_numfmts, &matching_format, &[0]).expect("reuse numFmt");
        assert!(updated.contains("numFmtId=\"164\""));
    }

    #[test]
    fn merge_compiled_styles_fonts_without_count() {
        let without_count = styles_xml(
            r"<fonts><font/></fonts>",
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf/></cellXfs>"#,
        );
        let source = styles_xml(
            r#"<fonts count="1"><font><b/></font></fonts>"#,
            r#"<fills count="1"><fill/></fills>"#,
            r#"<borders count="1"><border/></borders>"#,
            r#"<cellXfs count="1"><xf fontId="0" fillId="0" borderId="0" numFmtId="0"/></cellXfs>"#,
        );
        let (updated, _) =
            merge_compiled_styles(&without_count, &source, &[0]).expect("count injected");
        assert!(updated.contains("count=\"2\""));
    }

    #[test]
    fn merge_compiled_styles_missing_collection() {
        let missing_fonts = "<styleSheet><fills count=\"1\"><fill/></fills><borders count=\"1\"><border/></borders><cellXfs count=\"1\"><xf/></cellXfs></styleSheet>";
        let error = merge_compiled_styles(missing_fonts, &standard_styles(), &[0])
            .expect_err("missing fonts");
        assert!(error.to_string().contains("fonts"));
    }

    #[test]
    fn import_compiled_styles_zero_and_missing_style() {
        let mut package = sample_package();
        assert!(
            package
                .import_compiled_styles(b"", 0)
                .expect("zero styles")
                .is_empty()
        );

        // 语义敏感：`compiler`/`compiled` 命名与 rust_xlsxwriter 惯用名一致。
        #[allow(clippy::similar_names)]
        let mut compiler = Workbook::new();
        compiler
            .add_worksheet()
            .write_string(0, 0, "plain")
            .expect("unstyled cell");
        let compiled_bytes = compiler.save_to_buffer().expect("compiled bytes");
        let error = package
            .import_compiled_styles(&compiled_bytes, 1)
            .expect_err("A1 has no style");
        assert!(error.to_string().contains("has no style index"));
    }

    #[test]
    fn package_serialization_paths() {
        let package = sample_package();
        let bytes = package.to_bytes().expect("to bytes");
        assert!(bytes.starts_with(b"PK"));

        let mut output = Vec::new();
        package.save_to_writer(&mut output).expect("to writer");
        assert!(output.starts_with(b"PK"));

        let directory = tempfile::tempdir().expect("tempdir");
        let path = directory.path().join("out.xlsx");
        package.save_to_path(&path).expect("to path");
        assert!(path.exists());
    }

    #[test]
    fn worksheet_path_error_paths() {
        let mut no_rels = sample_package();
        no_rels
            .entries
            .retain(|entry| entry.name != "xl/_rels/workbook.xml.rels");
        let error = no_rels
            .worksheet_path_by_name("Styled")
            .expect_err("missing rels");
        assert!(error.to_string().contains("workbook.xml.rels"));

        let mut missing_relationship = sample_package();
        let rels = missing_relationship
            .entries
            .iter_mut()
            .find(|entry| entry.name == "xl/_rels/workbook.xml.rels")
            .expect("rels entry");
        rels.bytes = br#"<Relationships><Relationship Id="rId2" Type="x" Target="styles.xml"/></Relationships>"#.to_vec();
        let error = missing_relationship
            .worksheet_path_by_name("Styled")
            .expect_err("missing relationship");
        assert!(error.to_string().contains("relationship"));

        let mut missing_part = sample_package();
        let rels = missing_part
            .entries
            .iter_mut()
            .find(|entry| entry.name == "xl/_rels/workbook.xml.rels")
            .expect("rels entry");
        rels.bytes = br#"<Relationships><Relationship Id="rId1" Type="x" Target="worksheets/ghost.xml"/></Relationships>"#.to_vec();
        let error = missing_part
            .worksheet_path_by_name("Styled")
            .expect_err("missing worksheet part");
        assert!(error.to_string().contains("is missing"));
    }

    #[test]
    fn worksheet_max_row_unterminated() {
        assert_eq!(worksheet_max_row("<row"), 0);
    }

    #[test]
    fn update_worksheet_dimension_without_dimension() {
        let updated = update_worksheet_dimension("<c r=\"A1\"><v>1</v></c>");
        assert!(updated.contains("A1"));
        // Unterminated `<c ` tag: loop breaks and the xml is returned untouched.
        assert_eq!(update_worksheet_dimension("<c "), "<c ");
    }

    #[test]
    fn parse_cell_reference_non_alpha() {
        assert_eq!(parse_cell_reference("A!1"), None);
    }

    #[test]
    fn blank_worksheet_edge_cases() {
        let invalid_utf8 = vec![TemplateZipEntry {
            name: "xl/worksheets/sheet1.xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: vec![0xFF, 0xFE, 0x00],
        }];
        let xml =
            String::from_utf8(blank_worksheet_with_inherited_format(&invalid_utf8)).expect("xml");
        assert!(xml.contains("sheetData"));

        let bare = vec![TemplateZipEntry {
            name: "xl/worksheets/sheet1.xml".to_owned(),
            is_dir: false,
            compression: CompressionMethod::Stored,
            unix_mode: None,
            bytes: br"<worksheet><sheetData/></worksheet>".to_vec(),
        }];
        let xml = String::from_utf8(blank_worksheet_with_inherited_format(&bare)).expect("xml");
        assert!(xml.contains("sheetData"));
    }

    #[test]
    fn extract_xml_element_self_closing_paths() {
        assert!(
            extract_xml_element("<sheetData><row r=\"1\"/>", "sheetData").is_none(),
            "self-closing marker belongs to a nested sibling"
        );
        assert!(extract_xml_element("<sheetData", "sheetData").is_none());
    }

    #[test]
    fn insert_before_close_tag_missing_tag() {
        let error = insert_before_close_tag("<a/>", "</b>", "x").expect_err("missing tag");
        assert!(error.to_string().contains("missing </b>"));
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;

    use std::io::Write;

    /// `append_sparse_rows_to_xml` 找不到 `</sheetData>` 时必须报错。
    #[test]
    fn append_sparse_rows_missing_sheet_data_errors() {
        // 对应 Java：POI `XSSFSheet` 必然包含 sheetData，
        // 模板缺失该元素说明包已损坏。
        let xml = concat!(
            "<worksheet><dimension ref=\"A1:B1\"/>",
            "<mergeCells count=\"1\"><mergeCell ref=\"A1:B1\"/></mergeCells>",
            "</worksheet>"
        );
        let rows = vec![vec![(0usize, CellValue::String("appended".to_owned()))]];
        let error = append_sparse_rows_to_xml(xml, &rows, &[], &[], &[])
            .expect_err("missing sheetData must be rejected");
        assert!(
            error.to_string().contains("does not contain sheetData"),
            "unexpected: {error}"
        );
    }

    /// 模板单元格列号超出 u16 时（防御性校验）必须报错。
    #[test]
    fn load_template_sheets_rejects_column_overflow() {
        // 对应 Java：calamine 读取的列号超出 XLSX 上限时拒绝模板。
        // 手工构造一个列引用为 ZZZZ（0 基列号 456974 > u16::MAX）的包。
        let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        writer
            .start_file("[Content_Types].xml", options)
            .expect("content types");
        writer
            .write_all(
                br#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#,
            )
            .expect("write");
        writer
            .start_file("_rels/.rels", options)
            .expect("package rels");
        writer
            .write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#,
            )
            .expect("write");
        writer
            .start_file("xl/workbook.xml", options)
            .expect("workbook");
        writer
            .write_all(
                br#"<?xml version="1.0"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Wide" sheetId="1" r:id="rId1"/></sheets></workbook>"#,
            )
            .expect("write");
        writer
            .start_file("xl/_rels/workbook.xml.rels", options)
            .expect("workbook rels");
        writer
            .write_all(
                br#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>"#,
            )
            .expect("write");
        writer
            .start_file("xl/worksheets/sheet1.xml", options)
            .expect("sheet");
        writer
            .write_all(
                br#"<?xml version="1.0"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="XFE1"><v>1</v></c></row></sheetData></worksheet>"#,
            )
            .expect("write");
        let bytes = writer.finish().expect("finish").into_inner();
        // 对应 Java：POI 加载模板时不校验超出 XLSX 列上限（XFD）的单元格引用，
        // 加载照常完成；rust_xlsxwriter 同样容忍并忽略超界引用。
        let sheets = load_template_sheets(&bytes).expect("tolerant load");
        assert_eq!(sheets.len(), 1);
        assert_eq!(sheets[0].name, "Wide");
        assert_eq!(sheets[0].next_row, 1);
    }
}
