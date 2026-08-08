//! Template-backed workbook seeding for Java `withTemplate` + `doWrite`.
//!
//! **Default path:** ZIP/OOXML preserve ([`TemplatePackage`]). Clone the template
//! package, keep `xl/styles.xml` and worksheet `mergeCells` intact, append typed
//! rows into `sheetData`, and when a requested sheet is missing create a new
//! worksheet part without rewriting existing sheets.
//!
//! **Legacy path:** the `easyexcel-xlsx` engine supplies value-replay sheet
//! snapshots when callers explicitly set
//! [`crate::WriteOptions::use_legacy_template_seed`].

use std::io::Write;
use std::path::Path;

use crate::core::{CellValue, ExcelError, Result};

use easyexcel_xlsx::xlsx::OoxmlTemplatePackage;
use easyexcel_xlsx::xlsx::template_xml::{TemplateCellValue, TemplateMergeRange};

use crate::MergeRange;

/// Legacy value-replay snapshot owned by the XLSX engine.
pub(crate) use easyexcel_xlsx::LegacyTemplateSheet as TemplateSheetData;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 In-memory XLSX template package used by the ZIP preserve write path.
#[derive(Debug, Clone)]
pub(crate) struct TemplatePackage {
    entries: OoxmlTemplatePackage,
}

impl TemplatePackage {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Loads an XLSX template package from bytes.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::Format`] when the bytes are not a readable ZIP/OOXML package.
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        OoxmlTemplatePackage::from_bytes(bytes)
            .map(|entries| Self { entries })
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns worksheet names in workbook order.
    ///
    /// # Errors
    ///
    /// Returns a format error when workbook metadata cannot be parsed.
    pub(crate) fn sheet_names(&self) -> Result<Vec<String>> {
        self.entries.sheet_names().map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns the next zero-based append row for a worksheet name.
    ///
    /// # Errors
    ///
    /// Returns [`ExcelError::SheetNotFound`] when the sheet is absent.
    pub(crate) fn next_row_for_sheet(&self, sheet_name: &str) -> Result<u32> {
        self.entries
            .next_row_for_sheet(sheet_name)
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends typed rows into a worksheet's `sheetData`.
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends rows and applies optional per-row heights to the newly created
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends rows with optional row heights and per-cell workbook style indexes.
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Appends rows while preserving Java `null` row gaps without creating
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
            .append_rows(sheet_name, &rows, row_heights, cell_styles, absent_rows)
            .map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Applies column widths and absolute merged regions to one preserved
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Imports styles compiled by `rust_xlsxwriter` into the preserved
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Serializes the package to owned XLSX bytes.
    ///
    /// # Errors
    ///
    /// Returns a format or I/O error when ZIP writing fails.
    pub(crate) fn to_bytes(&self) -> Result<Vec<u8>> {
        self.entries.to_bytes().map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes the package to a filesystem path.
    ///
    /// # Errors
    ///
    /// Returns an I/O or format error.
    #[allow(dead_code)]
    pub(crate) fn save_to_path(&self, path: &Path) -> Result<()> {
        self.entries.save_to_path(path).map_err(ExcelError::from)
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Writes the package to an arbitrary writer.
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns whether [`crate::WriteOptions`] carries a template source.
///
/// Corresponds to Java `WriteWorkbook.templateFile` / `templateInputStream`
/// being non-null.
#[must_use]
pub(crate) fn has_template(template_file: Option<&Path>, template_bytes: Option<&[u8]>) -> bool {
    easyexcel_xlsx::has_template(template_file, template_bytes)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Loads template bytes from a file path or an in-memory copy.
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Rejects template types that Java also rejects for the XLSX ZIP path.
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Parses an XLSX template into ordered sheet snapshots.
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

/// 对应 Java：无直接对应对象；Rust 架构扩展。 Resolves a template target against a ZIP package sheet list.
#[must_use]
pub(crate) fn resolve_package_target(
    sheet_names: &[String],
    sheet_index: Option<usize>,
    sheet_name: &str,
) -> (usize, String, bool) {
    easyexcel_xlsx::resolve_sheet_target(sheet_names, sheet_index, sheet_name)
}

fn template_cell_value(value: &CellValue) -> Result<TemplateCellValue> {
    Ok(match value {
        CellValue::Empty | CellValue::Image(_) => TemplateCellValue::Empty,
        CellValue::String(text)
        | CellValue::Error(text)
        | CellValue::Hyperlink { text, .. }
        | CellValue::HyperlinkWithMetadata { text, .. } => TemplateCellValue::Text(text.clone()),
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
