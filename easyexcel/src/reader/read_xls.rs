//! Legacy XLS (BIFF) sheet enumeration and typed row dispatch via calamine.

use std::path::Path;

use calamine::{Reader, Xls, open_workbook};

use crate::core::{ExcelRow, ReadListener, Result};
use crate::reader::read_helpers::{format_error, reject_extra_read};
use crate::reader::read_options::ReadOptions;
use crate::reader::row_consumer::{ReadFlow, TypedRowConsumer};
use crate::reader::row_processing::{read_range, select_xls_sheets};
use crate::reader::xls_display::load_xls_displays;

/// Discovers worksheet names in workbook order.
///
/// 对应 Java：`XlsSaxAnalyser.sheetList()` via `XlsListSheetListener` /
/// calamine `Reader::sheet_names`.
///
/// # Errors
///
/// Returns an I/O or workbook-format error.
pub fn list_xls_sheets(path: &Path, options: &ReadOptions) -> Result<Vec<(usize, String)>> {
    reject_extra_read(options, "XLS")?;
    let workbook: Xls<_> = open_workbook(path).map_err(format_error)?;
    Ok(workbook.sheet_names().into_iter().enumerate().collect())
}

/// Reads selected legacy XLS sheets through the typed listener lifecycle.
///
/// Calamine materializes each XLS worksheet before row dispatch because the
/// binary BIFF format does not expose the XLSX cell-stream API.
///
/// # Errors
///
/// Returns an I/O, workbook-format, sheet-selection, conversion, or listener error.
pub fn read_xls<T, L>(path: &Path, options: &ReadOptions, listener: &mut L) -> Result<()>
where
    T: ExcelRow,
    L: ReadListener<T>,
{
    reject_extra_read(options, "XLS")?;
    let mut workbook: Xls<_> = open_workbook(path).map_err(format_error)?;
    let sheets = select_xls_sheets(workbook.worksheets(), &options.sheet, options.auto_trim)?;
    // Overlay BIFF FORMAT/XF display strings so STRING mode matches Java
    // BuiltinFormats (e.g. short date id 22 → `yyyy-m-d h:mm`).
    let displays = load_xls_displays(
        path,
        options.use_1904_windowing,
        &options.locale.formatter(),
    );
    for (sheet_no, sheet_name, range) in sheets {
        let mut consumer = TypedRowConsumer::<T> { listener };
        let sheet_displays = displays.get(sheet_no).cloned().unwrap_or_default();
        if read_range(
            &range,
            sheet_no,
            &sheet_name,
            options,
            &sheet_displays,
            &mut consumer,
        )? == ReadFlow::Stop
        {
            break;
        }
    }
    Ok(())
}
