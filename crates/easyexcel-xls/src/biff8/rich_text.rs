//! BIFF8 富文本记录关联与提取。

use std::collections::HashMap;
use std::path::Path;

use easyexcel_io::Result;

use super::{
    Biff8ContinuableRecordDecoder, Biff8ContinuableRecordKind, Biff8ContinuationStatus,
    Biff8DecodedContinuableRecord, Biff8Font, Biff8RichTextCell,
};

/// 读取工作簿中的富文本单元格，键为 `(sheet, row, column)`。
///
/// 这里完成 CFB/BIFF record 遍历、SST continuation 合并、LABELSST 引用关联和
/// FONT 解码；调用方只需把中立字体映射为自己的公开元数据。
pub fn load_rich_text_cells_with_password(
    path: &Path,
    password: Option<&str>,
) -> Result<HashMap<(usize, u32, usize), Biff8RichTextCell>> {
    let workbook = super::record_stream::read_workbook_stream_with_password(path, password)?;
    load_rich_text_cells(&workbook)
}

/// 从已解密的 BIFF8 Workbook stream 提取富文本单元格。
///
/// 调用方可与工作簿模型、数字显示和 extra record 解析共享同一缓冲区。
pub fn load_rich_text_cells(
    workbook: &[u8],
) -> Result<HashMap<(usize, u32, usize), Biff8RichTextCell>> {
    let mut decoder = Biff8ContinuableRecordDecoder::default();
    let mut shared_strings = Vec::new();
    let mut fonts = HashMap::new();
    let mut font_record_index = 0u16;
    let mut current_sheet = None;
    let mut next_sheet = 0usize;
    let mut references = Vec::new();

    super::record_stream::walk_biff_records(workbook, |sid, data| {
        if sid == super::record_sid::CONTINUE_SID {
            if decoder.push(data) {
                finish_rich_sst(&mut decoder, false, &mut shared_strings)?;
            }
            return Ok(());
        }
        finish_rich_sst(&mut decoder, true, &mut shared_strings)?;
        match sid {
            super::record_sid::FONT_SID if current_sheet.is_none() => {
                // BIFF8 为兼容历史 HSSF 字体索引跳过逻辑编号 4。
                let logical_index = if font_record_index >= 4 {
                    font_record_index.saturating_add(1)
                } else {
                    font_record_index
                };
                if let Some(font) = Biff8Font::decode(data) {
                    fonts.insert(logical_index, font);
                }
                font_record_index = font_record_index.saturating_add(1);
            }
            super::record_sid::BOF_SID => {
                if let Some(kind) = super::event_record::decode_bof_type(data) {
                    match kind {
                        super::event_record::Biff8BofType::Workbook => current_sheet = None,
                        super::event_record::Biff8BofType::Worksheet => {
                            current_sheet = Some(next_sheet);
                            next_sheet = next_sheet.saturating_add(1);
                        }
                        super::event_record::Biff8BofType::Other(_) => {}
                    }
                }
            }
            super::record_sid::SST_SID => {
                decoder.begin(Biff8ContinuableRecordKind::SharedStringTable, data);
                finish_rich_sst(&mut decoder, false, &mut shared_strings)?;
            }
            super::record_sid::LABEL_SST_SID => {
                if let (Some(sheet), Some(record)) = (
                    current_sheet,
                    super::event_record::decode_label_sst_record(data),
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
        let mut runs = Vec::new();
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
                runs.push((start, end, font.clone()));
            }
        }
        if !runs.is_empty() {
            cells.insert(
                (sheet, row, usize::from(column)),
                Biff8RichTextCell::new(value.text.clone(), runs),
            );
        }
    }
    Ok(cells)
}

fn finish_rich_sst(
    decoder: &mut Biff8ContinuableRecordDecoder,
    require_complete: bool,
    shared_strings: &mut Vec<crate::Biff8SstString>,
) -> Result<()> {
    if let Biff8ContinuationStatus::Complete(Biff8DecodedContinuableRecord::SharedStrings(
        strings,
    )) = decoder.try_finish(require_complete)?
    {
        *shared_strings = strings;
    }
    Ok(())
}
