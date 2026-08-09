//! Legacy XLS (BIFF) sheet enumeration and typed row dispatch.

use std::collections::HashMap;
use std::path::Path;

use crate::analysis::v03::XlsRecordDispatcher;
use crate::core::{
    CellExtra, ExcelColor, ExcelError, ExcelFontScript, ExcelRow, ExcelUnderline, ReadListener,
    Result, RichTextStringData, WriteFont,
};
use crate::read::read_helpers::validate_read_options;
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
    validate_read_options(options)?;
    let workbook = easyexcel_xls::read_path_with_password(path, options.password.as_deref())?;
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
    validate_read_options(options)?;
    // 只打开/解密一次 OLE2 Workbook stream；模型、格式化显示、富文本与 extra
    // 事件在同一借用缓冲区上完成各自的顺序扫描。
    let workbook_stream = easyexcel_xls::biff8::record_stream::read_workbook_stream_with_password(
        path,
        options.password.as_deref(),
    )?;
    let workbook = easyexcel_xls::read_decrypted_workbook_stream(&workbook_stream)?;
    let extras = load_xls_extras(&workbook_stream, options)?;
    let rich_text_cells = load_xls_rich_text_cells(&workbook_stream)?;
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
    let displays = easyexcel_xls::biff8::format_numeric_displays(
        &workbook_stream,
        options.use_1904_windowing,
        &options.locale.formatter(),
    );
    let empty_sheet_displays = HashMap::new();
    for (sheet_no, sheet_name) in sheets {
        let mut consumer = TypedRowConsumer::<T> { listener };
        let sheet_displays = displays.get(sheet_no).unwrap_or(&empty_sheet_displays);
        let sheet_extras = extras
            .iter()
            .filter_map(|(extra_sheet, extra)| (*extra_sheet == sheet_no).then_some(extra))
            .collect::<Vec<_>>();
        if read_model_sheet(
            &workbook.sheets[sheet_no],
            sheet_no,
            &sheet_name,
            options,
            sheet_displays,
            &rich_text_cells,
            &sheet_extras,
            &mut consumer,
        )? == ReadFlow::Stop
        {
            break;
        }
    }
    Ok(())
}

fn load_xls_rich_text_cells(
    workbook_stream: &[u8],
) -> Result<HashMap<(usize, u32, usize), RichTextStringData>> {
    easyexcel_xls::biff8::load_rich_text_cells(workbook_stream)
        .map(|cells| {
            cells
                .into_iter()
                .map(|(position, cell)| {
                    let intervals = cell.runs().iter().map(|(start, end, font)| {
                        crate::core::IntervalFont::new(*start, *end, facade_font(font))
                    });
                    (
                        position,
                        RichTextStringData::new(cell.text()).interval_font_list(intervals),
                    )
                })
                .collect()
        })
        .map_err(ExcelError::from)
}

fn facade_font(engine_font: &easyexcel_xls::biff8::Biff8Font) -> WriteFont {
    let mut facade_font = WriteFont::new()
        .font_height_in_points(f64::from(engine_font.height_twips()) / 20.0)
        .italic(engine_font.italic())
        .strikeout(engine_font.strikeout())
        .bold(engine_font.bold())
        .charset(engine_font.charset());
    if !engine_font.name().is_empty() {
        facade_font = facade_font.font_name(engine_font.name());
    }
    if let Some(index) = engine_font.color_index() {
        facade_font = facade_font.color(ExcelColor::Indexed(index));
    }
    facade_font = match engine_font.script() {
        1 => facade_font.type_offset(ExcelFontScript::Superscript),
        2 => facade_font.type_offset(ExcelFontScript::Subscript),
        _ => facade_font.type_offset(ExcelFontScript::None),
    };
    match engine_font.underline() {
        1 => facade_font.underline(ExcelUnderline::Single),
        2 => facade_font.underline(ExcelUnderline::Double),
        0x21 => facade_font.underline(ExcelUnderline::SingleAccounting),
        0x22 => facade_font.underline(ExcelUnderline::DoubleAccounting),
        _ => facade_font.underline(ExcelUnderline::None),
    }
}

fn load_xls_extras(
    workbook_stream: &[u8],
    options: &ReadOptions,
) -> Result<Vec<(usize, CellExtra)>> {
    if options.extra_read.is_empty() {
        return Ok(Vec::new());
    }
    let mut dispatcher = XlsRecordDispatcher::new(options);
    easyexcel_xls::biff8::record_stream::walk_biff_records(workbook_stream, |record_sid, data| {
        dispatcher
            .process_record(record_sid, data)
            .map_err(|error| easyexcel_io::Error::Other(error.to_string()))
    })
    .map_err(ExcelError::from)?;
    dispatcher.finish_records()?;
    Ok(dispatcher.state().extras().to_vec())
}
