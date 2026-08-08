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
    let workbook = easyexcel_xls::read_path_with_password(path, options.password.as_deref())?;
    let extras = load_xls_extras(path, options)?;
    let rich_text_cells = load_xls_rich_text_cells(path, options)?;
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
    let displays = easyexcel_xls::biff8::load_numeric_displays_with_password(
        path,
        options.use_1904_windowing,
        &options.locale.formatter(),
        options.password.as_deref(),
    )
    .unwrap_or_default();
    for (sheet_no, sheet_name) in sheets {
        let mut consumer = TypedRowConsumer::<T> { listener };
        let sheet_displays = displays.get(sheet_no).cloned().unwrap_or_default();
        let sheet_extras = extras
            .iter()
            .filter_map(|(extra_sheet, extra)| (*extra_sheet == sheet_no).then_some(extra))
            .collect::<Vec<_>>();
        if read_model_sheet(
            &workbook.sheets[sheet_no],
            sheet_no,
            &sheet_name,
            options,
            &sheet_displays,
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
    path: &Path,
    options: &ReadOptions,
) -> Result<HashMap<(usize, u32, usize), RichTextStringData>> {
    use easyexcel_xls::biff8::{Biff8ContinuableRecordDecoder, Biff8ContinuableRecordKind};

    let workbook = easyexcel_xls::biff8::record_stream::read_workbook_stream_with_password(
        path,
        options.password.as_deref(),
    )?;
    let mut decoder = Biff8ContinuableRecordDecoder::default();
    let mut shared_strings = Vec::new();
    let mut fonts = HashMap::new();
    let mut font_record_index = 0u16;
    let mut current_sheet = None;
    let mut next_sheet = 0usize;
    let mut references = Vec::new();

    easyexcel_xls::biff8::record_stream::walk_biff_records(&workbook, |sid, data| {
        if sid == easyexcel_xls::biff8::record_sid::CONTINUE_SID {
            if decoder.push(data) {
                finish_rich_sst(&mut decoder, false, &mut shared_strings)?;
            }
            return Ok(());
        }
        finish_rich_sst(&mut decoder, true, &mut shared_strings)?;
        match sid {
            easyexcel_xls::biff8::record_sid::FONT_SID if current_sheet.is_none() => {
                let logical_index = if font_record_index >= 4 {
                    font_record_index.saturating_add(1)
                } else {
                    font_record_index
                };
                if let Some(font) = decode_biff8_font(data) {
                    fonts.insert(logical_index, font);
                }
                font_record_index = font_record_index.saturating_add(1);
            }
            easyexcel_xls::biff8::record_sid::BOF_SID => {
                if let Some(kind) = easyexcel_xls::biff8::event_record::decode_bof_type(data) {
                    match kind {
                        easyexcel_xls::biff8::event_record::Biff8BofType::Workbook => {
                            current_sheet = None;
                        }
                        easyexcel_xls::biff8::event_record::Biff8BofType::Worksheet => {
                            current_sheet = Some(next_sheet);
                            next_sheet = next_sheet.saturating_add(1);
                        }
                        easyexcel_xls::biff8::event_record::Biff8BofType::Other(_) => {}
                    }
                }
            }
            easyexcel_xls::biff8::record_sid::SST_SID => {
                decoder.begin(Biff8ContinuableRecordKind::SharedStringTable, data);
                finish_rich_sst(&mut decoder, false, &mut shared_strings)?;
            }
            easyexcel_xls::biff8::record_sid::LABEL_SST_SID => {
                if let (Some(sheet), Some(record)) = (
                    current_sheet,
                    easyexcel_xls::biff8::event_record::decode_label_sst_record(data),
                ) {
                    references.push((
                        sheet,
                        record.header.row,
                        record.header.column,
                        record.sst_index,
                    ));
                }
            }
            _ => {}
        }
        Ok(())
    })?;
    finish_rich_sst(&mut decoder, true, &mut shared_strings)?;

    let mut cells = HashMap::new();
    for (sheet, row, column, sst_index) in references {
        let Some(value) = shared_strings.get(sst_index) else {
            continue;
        };
        let utf16_len = value.text.encode_utf16().count();
        let mut intervals = Vec::new();
        for (index, &(start, font_index)) in value.formatting_runs.iter().enumerate() {
            let start = usize::from(start);
            let end = value
                .formatting_runs
                .get(index.saturating_add(1))
                .map_or(utf16_len, |run| usize::from(run.0));
            if start >= end || end > utf16_len {
                continue;
            }
            if let Some(font) = fonts.get(&font_index) {
                intervals.push(crate::core::IntervalFont::new(start, end, font.clone()));
            }
        }
        if !intervals.is_empty() {
            cells.insert(
                (sheet, row, column),
                RichTextStringData::new(value.text.clone()).interval_font_list(intervals),
            );
        }
    }
    Ok(cells)
}

fn finish_rich_sst(
    decoder: &mut easyexcel_xls::biff8::Biff8ContinuableRecordDecoder,
    require_complete: bool,
    shared_strings: &mut Vec<easyexcel_xls::Biff8SstString>,
) -> easyexcel_io::Result<()> {
    if let easyexcel_xls::biff8::Biff8ContinuationStatus::Complete(
        easyexcel_xls::biff8::Biff8DecodedContinuableRecord::SharedStrings(strings),
    ) = decoder.try_finish(require_complete)?
    {
        *shared_strings = strings;
    }
    Ok(())
}

fn decode_biff8_font(data: &[u8]) -> Option<WriteFont> {
    if data.len() < 16 {
        return None;
    }
    let height_twips = u16::from_le_bytes([data[0], data[1]]);
    let options = u16::from_le_bytes([data[2], data[3]]);
    let color_index = u16::from_le_bytes([data[4], data[5]]);
    let weight = u16::from_le_bytes([data[6], data[7]]);
    let script = u16::from_le_bytes([data[8], data[9]]);
    let name_len = usize::from(data[14]);
    let wide = data[15] & 0x01 != 0;
    let name = if wide {
        let bytes = data.get(16..16usize.saturating_add(name_len.saturating_mul(2)))?;
        let units = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    } else {
        data.get(16..16usize.saturating_add(name_len))?
            .iter()
            .map(|byte| char::from(*byte))
            .collect()
    };
    let mut font = WriteFont::new()
        .font_height_in_points(f64::from(height_twips) / 20.0)
        .italic(options & 0x0002 != 0)
        .strikeout(options & 0x0008 != 0)
        .bold(weight >= 700)
        .charset(data[12]);
    if !name.is_empty() {
        font = font.font_name(name);
    }
    if let Ok(index) = u8::try_from(color_index)
        && index <= 64
    {
        font = font.color(ExcelColor::Indexed(index));
    }
    font = match script {
        1 => font.type_offset(ExcelFontScript::Superscript),
        2 => font.type_offset(ExcelFontScript::Subscript),
        _ => font.type_offset(ExcelFontScript::None),
    };
    font = match data[10] {
        1 => font.underline(ExcelUnderline::Single),
        2 => font.underline(ExcelUnderline::Double),
        0x21 => font.underline(ExcelUnderline::SingleAccounting),
        0x22 => font.underline(ExcelUnderline::DoubleAccounting),
        _ => font.underline(ExcelUnderline::None),
    };
    Some(font)
}

fn load_xls_extras(path: &Path, options: &ReadOptions) -> Result<Vec<(usize, CellExtra)>> {
    if options.extra_read.is_empty() {
        return Ok(Vec::new());
    }
    let workbook = easyexcel_xls::biff8::record_stream::read_workbook_stream_with_password(
        path,
        options.password.as_deref(),
    )?;
    let mut dispatcher = XlsRecordDispatcher::new(options);
    easyexcel_xls::biff8::record_stream::walk_biff_records(&workbook, |record_sid, data| {
        dispatcher
            .process_record(record_sid, data)
            .map_err(|error| easyexcel_io::Error::Other(error.to_string()))
    })
    .map_err(ExcelError::from)?;
    dispatcher.finish_records()?;
    Ok(dispatcher.state().extras().to_vec())
}
