//! Explicit legacy XLSX template value replay.
//!
//! This opt-in path replays values into a fresh `rust_xlsxwriter` workbook and
//! therefore does not preserve styles or unknown OOXML parts. The default
//! round-trip path is [`super::OoxmlTemplatePackage`].

use std::io::Cursor;

use easyexcel_io::{Error, Result};
use easyexcel_model::Cell;
use rust_xlsxwriter::{Workbook, Worksheet};

/// One worksheet value snapshot used by the opt-in legacy template path.
#[derive(Debug, Clone)]
pub struct LegacyTemplateSheet {
    /// Worksheet name in workbook order.
    pub name: String,
    /// Non-empty cells with zero-based coordinates.
    pub cells: Vec<(u32, u16, Cell)>,
    /// Next zero-based row available for append.
    pub next_row: u32,
}

/// Parse an XLSX package into neutral value snapshots.
///
/// # Errors
///
/// Returns a format error for unreadable packages, workbooks without sheets,
/// or column indexes that do not fit the XLSX writer API.
pub fn load_legacy_template_sheets(bytes: &[u8]) -> Result<Vec<LegacyTemplateSheet>> {
    let workbook = super::read(Cursor::new(bytes))?;
    if workbook.sheets.is_empty() {
        return Err(Error::Xlsx(
            "withTemplate workbook contains no worksheets".to_owned(),
        ));
    }
    workbook
        .sheets
        .into_iter()
        .map(|sheet| {
            let mut cells = Vec::with_capacity(sheet.cells.len());
            let mut last_row = None;
            for ((row, column), cell) in sheet.cells {
                if cell.is_empty() {
                    continue;
                }
                let column = u16::try_from(column).map_err(|_| {
                    Error::Xlsx(format!(
                        "withTemplate sheet `{}` column index {column} exceeds u16",
                        sheet.name
                    ))
                })?;
                last_row = Some(last_row.map_or(row, |current: u32| current.max(row)));
                cells.push((row, column, cell));
            }
            Ok(LegacyTemplateSheet {
                name: sheet.name,
                cells,
                next_row: last_row.map_or(0, |row| row.saturating_add(1)),
            })
        })
        .collect()
}

/// Replay neutral template values into a fresh `rust_xlsxwriter` workbook.
///
/// # Errors
///
/// Returns an XLSX error when worksheet creation or value writing fails.
pub fn seed_legacy_template_workbook(
    workbook: &mut Workbook,
    sheets: &[LegacyTemplateSheet],
) -> Result<()> {
    for sheet in sheets {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(&sheet.name)
            .map_err(|error| Error::Xlsx(error.to_string()))?;
        for (row, column, value) in &sheet.cells {
            write_legacy_template_cell(worksheet, *row, *column, value)?;
        }
    }
    Ok(())
}

fn write_legacy_template_cell(
    worksheet: &mut Worksheet,
    row: u32,
    column: u16,
    value: &Cell,
) -> Result<()> {
    let result = match value {
        Cell::Empty => return Ok(()),
        Cell::Text(text) => worksheet.write_string(row, column, text).map(|_| ()),
        Cell::Bool(value) => worksheet.write_boolean(row, column, *value).map(|_| ()),
        Cell::Number(value) => worksheet.write_number(row, column, *value).map(|_| ()),
        Cell::Error(error) => worksheet
            .write_string(row, column, error.to_string())
            .map(|_| ()),
        Cell::Formula { expr, .. } => worksheet
            .write_formula(row, column, expr.as_str())
            .map(|_| ()),
    };
    result.map_err(|error| Error::Xlsx(error.to_string()))
}
