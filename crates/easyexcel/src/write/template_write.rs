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
use easyexcel_xlsx::{TemplateDecoration, TemplateDecorationPlacement};

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
        let snapshot = self.clone();
        match self.append_rows_with_layout_and_absent_inner(
            sheet_name,
            rows,
            row_heights,
            cell_styles,
            absent_rows,
        ) {
            Ok(next_row) => Ok(next_row),
            Err(error) => {
                *self = snapshot;
                Err(error)
            }
        }
    }

    fn append_rows_with_layout_and_absent_inner(
        &mut self,
        sheet_name: &str,
        rows: &[Vec<(usize, CellValue)>],
        row_heights: &[Option<u16>],
        cell_styles: &[Vec<Option<u32>>],
        absent_rows: &[bool],
    ) -> Result<u32> {
        let first_row = self.next_row_for_sheet(sheet_name)?;
        let mut mapped_rows = Vec::with_capacity(rows.len());
        let mut decorations = Vec::new();
        for (row_offset, row) in rows.iter().enumerate() {
            let physical_row = first_row
                .saturating_add(u32::try_from(row_offset).unwrap_or(u32::MAX));
            let mut mapped = Vec::with_capacity(row.len());
            for (column, value) in row {
                let column_index = u16::try_from(*column).map_err(|_| {
                    ExcelError::Format("XLSX template column index exceeds u16".to_owned())
                })?;
                let value = template_cell_value(value)?;
                decorations.extend(easyexcel_xlsx::template_value_decorations(
                    &value,
                    physical_row,
                    column_index,
                ));
                mapped.push((*column, value));
            }
            mapped_rows.push(mapped);
        }
        let next_row = self.entries
            .append_rows(
                sheet_name,
                &mapped_rows,
                row_heights,
                cell_styles,
                absent_rows,
            )
            .map_err(ExcelError::from)?;
        self.apply_template_decorations(sheet_name, decorations)?;
        Ok(next_row)
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

    /// 在保留未知 OOXML 部件的前提下新增或替换 typed 单元格及全部装饰。
    pub(crate) fn set_cell_with_decorations(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
        value: &TemplateCellValue,
    ) -> Result<()> {
        let snapshot = self.clone();
        let result = (|| -> Result<()> {
            self.entries
                .set_cell(sheet_name, row_index, column_index, value)
                .map_err(ExcelError::from)?;
            self.apply_template_decorations(
                sheet_name,
                easyexcel_xlsx::template_value_decorations(value, row_index, column_index),
            )
        })();
        if result.is_err() {
            *self = snapshot;
        }
        result
    }

    /// 删除模板工作表中指定单元格的传统批注。
    pub(crate) fn remove_comment(
        &mut self,
        sheet_name: &str,
        row_index: u32,
        column_index: u16,
    ) -> Result<bool> {
        self.entries
            .remove_comment(sheet_name, row_index, column_index)
            .map_err(ExcelError::from)
    }

    /// 为指定模板工作表写入 OOXML 工作表保护。
    pub(crate) fn protect_sheet(&mut self, sheet_name: &str, password: &str) -> Result<()> {
        self.entries
            .protect_sheet(sheet_name, password)
            .map_err(ExcelError::from)
    }

    fn apply_template_decorations(
        &mut self,
        sheet_name: &str,
        decorations: Vec<TemplateDecorationPlacement>,
    ) -> Result<()> {
        for placement in decorations {
            match placement.decoration {
                TemplateDecoration::Comment(comment) => self
                    .entries
                    .set_template_comment(
                        sheet_name,
                        placement.row,
                        placement.column,
                        &comment,
                    )
                    .map_err(ExcelError::from)?,
                TemplateDecoration::Hyperlink(hyperlink) => self
                    .entries
                    .set_template_hyperlink(
                        sheet_name,
                        placement.row,
                        placement.column,
                        &hyperlink,
                    )
                    .map_err(ExcelError::from)?,
                TemplateDecoration::Image(image) => self
                    .entries
                    .set_template_image(
                        sheet_name,
                        placement.row,
                        placement.column,
                        &image,
                    )
                    .map_err(ExcelError::from)?,
            }
        }
        Ok(())
    }

    /// 将编译工作簿中的图表部件导入指定模板工作表。
    pub(crate) fn import_chart(&mut self, compiled_xlsx: &[u8], sheet_name: &str) -> Result<()> {
        self.entries
            .import_chart(compiled_xlsx, sheet_name)
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

pub(crate) fn template_cell_value(value: &CellValue) -> Result<TemplateCellValue> {
    Ok(match value {
        CellValue::Empty => TemplateCellValue::Empty,
        CellValue::Image(bytes) => TemplateCellValue::Images {
            value: Box::new(TemplateCellValue::Empty),
            images: vec![easyexcel_xlsx::TemplateImage::new(bytes.clone())],
        },
        CellValue::String(text) | CellValue::Error(text) => TemplateCellValue::Text(text.clone()),
        CellValue::Hyperlink { url, text } => template_hyperlink_value(
            url,
            text,
            crate::HyperlinkType::Url,
            crate::CoordinateData::new(),
        ),
        CellValue::HyperlinkWithMetadata {
            address,
            text,
            hyperlink_type,
            coordinates,
        } => {
            if *hyperlink_type == crate::HyperlinkType::None {
                TemplateCellValue::Text(text.clone())
            } else {
                template_hyperlink_value(address, text, *hyperlink_type, *coordinates)
            }
        }
        CellValue::RichText(rich) => {
            return crate::write::excel_writer_core::template_rich_text_cell_value(rich);
        }
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
        CellValue::Comment { value, text } => TemplateCellValue::Comment {
            value: Box::new(template_cell_value(value)?),
            comment: easyexcel_xlsx::TemplateComment {
                text: text.clone(),
                ..easyexcel_xlsx::TemplateComment::default()
            },
        },
        CellValue::CommentWithMetadata { value, comment } => TemplateCellValue::Comment {
            value: Box::new(template_cell_value(value)?),
            comment: template_comment_data(comment),
        },
        CellValue::Images { value, images } => {
            return template_images_value(value, images);
        }
    })
}

pub(crate) fn template_hyperlink_value(
    address: &str,
    text: &str,
    hyperlink_type: crate::HyperlinkType,
    coordinates: crate::CoordinateData,
) -> TemplateCellValue {
    let engine_type = template_hyperlink_type(hyperlink_type);
    let mut hyperlink = easyexcel_xlsx::TemplateHyperlink::new(address, engine_type);
    hyperlink.first_row = template_hyperlink_coordinate(
        coordinates.get_first_row_index(),
        coordinates.get_relative_first_row_index(),
    );
    hyperlink.first_column = template_hyperlink_coordinate(
        coordinates.get_first_column_index().map(u32::from),
        coordinates.get_relative_first_column_index(),
    );
    hyperlink.last_row = template_hyperlink_coordinate(
        coordinates.get_last_row_index(),
        coordinates.get_relative_last_row_index(),
    );
    hyperlink.last_column = template_hyperlink_coordinate(
        coordinates.get_last_column_index().map(u32::from),
        coordinates.get_relative_last_column_index(),
    );
    TemplateCellValue::Hyperlink {
        value: Box::new(TemplateCellValue::Text(text.to_owned())),
        hyperlink,
    }
}

/// 将 Java 兼容超链接类型转换为 XLSX 引擎类型。
#[must_use]
pub(crate) const fn template_hyperlink_type(
    hyperlink_type: crate::HyperlinkType,
) -> easyexcel_xlsx::TemplateHyperlinkType {
    match hyperlink_type {
        crate::HyperlinkType::None | crate::HyperlinkType::Url => {
            easyexcel_xlsx::TemplateHyperlinkType::Url
        }
        crate::HyperlinkType::Document => easyexcel_xlsx::TemplateHyperlinkType::Document,
        crate::HyperlinkType::Email => easyexcel_xlsx::TemplateHyperlinkType::Email,
        crate::HyperlinkType::File => easyexcel_xlsx::TemplateHyperlinkType::File,
    }
}

const fn template_hyperlink_coordinate(
    absolute: Option<u32>,
    relative: Option<i32>,
) -> easyexcel_xlsx::TemplateHyperlinkCoordinate {
    easyexcel_xlsx::TemplateHyperlinkCoordinate { absolute, relative }
}

pub(crate) fn template_images_value(
    value: &CellValue,
    images: &[crate::ImageData],
) -> Result<TemplateCellValue> {
    Ok(TemplateCellValue::Images {
        value: Box::new(template_cell_value(value)?),
        images: images.iter().map(template_image_data).collect(),
    })
}

fn template_image_data(image: &crate::ImageData) -> easyexcel_xlsx::TemplateImage {
    let anchor = image.get_anchor();
    let coordinates = anchor.get_coordinates();
    let movement = match anchor
        .get_anchor_type()
        .unwrap_or(crate::AnchorType::MoveAndResize)
    {
        crate::AnchorType::MoveAndResize => easyexcel_xlsx::TemplateImageMovement::MoveAndResize,
        crate::AnchorType::DontMoveDoResize | crate::AnchorType::MoveDontResize => {
            easyexcel_xlsx::TemplateImageMovement::MoveDontResize
        }
        crate::AnchorType::DontMoveAndResize => {
            easyexcel_xlsx::TemplateImageMovement::DontMoveOrResize
        }
    };
    easyexcel_xlsx::TemplateImage {
        bytes: image.image().to_vec(),
        first_row: easyexcel_xlsx::AnchorCoordinate {
            absolute: coordinates.get_first_row_index(),
            relative: coordinates.get_relative_first_row_index(),
        },
        first_column: easyexcel_xlsx::AnchorCoordinate {
            absolute: coordinates.get_first_column_index().map(u32::from),
            relative: coordinates.get_relative_first_column_index(),
        },
        last_row: easyexcel_xlsx::AnchorCoordinate {
            absolute: coordinates.get_last_row_index(),
            relative: coordinates.get_relative_last_row_index(),
        },
        last_column: easyexcel_xlsx::AnchorCoordinate {
            absolute: coordinates.get_last_column_index().map(u32::from),
            relative: coordinates.get_relative_last_column_index(),
        },
        left: anchor.get_left().unwrap_or(0),
        right: anchor.get_right().unwrap_or(0),
        top: anchor.get_top().unwrap_or(0),
        bottom: anchor.get_bottom().unwrap_or(0),
        movement,
    }
}

pub(crate) fn template_comment_data(
    comment: &crate::CommentData,
) -> easyexcel_xlsx::TemplateComment {
    let movement = match comment.get_anchor().get_anchor_type() {
        Some(crate::AnchorType::MoveAndResize)
        | Some(crate::AnchorType::DontMoveDoResize) => Some(0),
        Some(crate::AnchorType::MoveDontResize) => Some(1),
        Some(crate::AnchorType::DontMoveAndResize) => Some(2),
        None => None,
    };
    easyexcel_xlsx::TemplateComment {
        text: comment.note_text(),
        author: comment.get_author().map(str::to_owned),
        movement,
        visible: comment.get_visible(),
    }
}
