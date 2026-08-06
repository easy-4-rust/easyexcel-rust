//! Legacy XLS (BIFF) sheet enumeration and typed row dispatch.

use std::path::Path;

use crate::core::{ExcelRow, ReadListener, Result};
use crate::read::read_helpers::reject_extra_read;
use crate::read::read_options::ReadOptions;
use crate::read::row_consumer::{ReadFlow, TypedRowConsumer};
use crate::read::row_processing::{read_model_sheet, select_sheet_names};

/// Discovers worksheet names in workbook order.
///
/// 对应 Java：`XlsSaxAnalyser.sheetList()` via `XlsListSheetListener` /
/// `easyexcel-xls` 工作簿模型中的工作表顺序。
///
/// # Errors
///
/// Returns an I/O or workbook-format error.
pub fn list_xls_sheets(path: &Path, options: &ReadOptions) -> Result<Vec<(usize, String)>> {
    reject_extra_read(options, "XLS")?;
    let workbook = easyexcel_xls::read_path(path)?;
    Ok(workbook
        .sheets
        .into_iter()
        .map(|sheet| sheet.name)
        .enumerate()
        .collect())
}

/// 对应 Java：`XlsSaxAnalyser.sheetList()`。 Reads selected legacy XLS sheets through the typed listener lifecycle.
///
/// `easyexcel-xls` 负责 BIFF/CFB 解析，本门面仅将中立工作表送入 listener 生命周期。
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
    let workbook = easyexcel_xls::read_path(path)?;
    let sheets = select_sheet_names(
        workbook
            .sheets
            .iter()
            .map(|sheet| sheet.name.clone())
            .collect(),
        &options.sheet,
        options.auto_trim,
    )?;
    // Overlay BIFF FORMAT/XF display strings so STRING mode matches Java
    // BuiltinFormats (e.g. short date id 22 → `yyyy-m-d h:mm`).
    let displays = easyexcel_xls::biff8::load_numeric_displays(
        path,
        options.use_1904_windowing,
        &options.locale.formatter(),
    )
    .unwrap_or_default();
    for (sheet_no, sheet_name) in sheets {
        let mut consumer = TypedRowConsumer::<T> { listener };
        let sheet_displays = displays.get(sheet_no).cloned().unwrap_or_default();
        if read_model_sheet(
            &workbook.sheets[sheet_no],
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
