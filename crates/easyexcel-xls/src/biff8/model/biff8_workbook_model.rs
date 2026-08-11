use easyexcel_io::{Error as ExcelError, Result};

use super::{
    Biff8Globals, Biff8ObjectModel, Biff8Record, Biff8WorksheetModel, RecordSink, RecordTransform,
};
use crate::biff8::encode::{BOF, BOUNDSHEET, DT_WORKSHEET, EOF, MSODRAWING, OBJ, TXO};

/// 可变 BIFF8 workbook record 模型。
///
/// 对应 Java：POI `HSSFWorkbook` 的 `InternalWorkbook + InternalSheet` 底座。
/// 已知子流被拆成 typed 容器，未知记录仍保留 SID、payload 与相对顺序。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8WorkbookModel {
    globals: Biff8Globals,
    worksheets: Vec<Biff8WorksheetModel>,
}

impl Biff8WorkbookModel {
    /// 从完整 Workbook stream 字节建立可变模型。
    ///
    /// # Errors
    ///
    /// 记录帧损坏或子流结构不闭合时返回错误。
    pub fn from_workbook_stream(workbook: &[u8]) -> Result<Self> {
        let mut records = Vec::new();
        crate::biff8::record_stream::walk_biff_records(workbook, |sid, payload| {
            records.push(Biff8Record::new(sid, payload.to_vec()));
            Ok(())
        })?;
        Self::from_records(records)
    }

    /// 从平铺的 Workbook stream records 建立模型。
    ///
    /// # Errors
    ///
    /// BOF/EOF 不闭合、BoundSheet 数量与顶层 sheet 子流不一致或名称损坏时
    /// 返回错误。
    pub fn from_records(records: Vec<Biff8Record>) -> Result<Self> {
        let spans = top_level_substreams(&records)?;
        let Some((globals_start, globals_end)) = spans.first().copied() else {
            return Err(ExcelError::Xls(
                "BIFF8 Workbook stream has no globals substream".to_owned(),
            ));
        };
        if globals_start != 0 {
            return Err(ExcelError::Xls(
                "BIFF8 Workbook stream contains records before globals BOF".to_owned(),
            ));
        }
        let globals_records = records[globals_start..=globals_end].to_vec();
        let names = globals_records
            .iter()
            .filter(|record| record.sid() == BOUNDSHEET)
            .map(|record| decode_boundsheet_name(record.payload()))
            .collect::<Result<Vec<_>>>()?;
        let sheet_spans = &spans[1..];
        if names.len() != sheet_spans.len() {
            return Err(ExcelError::Xls(format!(
                "BOUNDSHEET count ({}) does not match top-level sheet substream count ({})",
                names.len(),
                sheet_spans.len()
            )));
        }

        let mut worksheets = Vec::with_capacity(sheet_spans.len());
        let mut next_object_id = 1u16;
        for (bound_sheet_index, ((start, end), name)) in
            sheet_spans.iter().copied().zip(names).enumerate()
        {
            let sheet_records = records[start..=end].to_vec();
            let object_records = sheet_records
                .iter()
                .filter(|record| matches!(record.sid(), MSODRAWING | OBJ | TXO))
                .cloned()
                .collect::<Vec<_>>();
            for record in sheet_records.iter().filter(|record| record.sid() == OBJ) {
                if record.payload().len() >= 8 {
                    let object_id = u16::from_le_bytes([record.payload()[6], record.payload()[7]]);
                    next_object_id = next_object_id.max(object_id.saturating_add(1));
                }
            }
            let is_worksheet = sheet_records
                .first()
                .is_some_and(|record| is_worksheet_bof(record.payload()));
            worksheets.push(Biff8WorksheetModel::new(
                name,
                bound_sheet_index,
                is_worksheet,
                sheet_records,
                Biff8ObjectModel::new(object_records, next_object_id),
            ));
        }

        Ok(Self {
            globals: Biff8Globals::new(globals_records),
            worksheets,
        })
    }

    /// 返回 globals 模型。
    #[must_use]
    pub const fn globals(&self) -> &Biff8Globals {
        &self.globals
    }

    /// 返回可变 globals 模型。
    #[must_use]
    pub fn globals_mut(&mut self) -> &mut Biff8Globals {
        &mut self.globals
    }

    /// 返回全部 BoundSheet 子流，包括 chart/macro sheet。
    #[must_use]
    pub fn worksheets(&self) -> &[Biff8WorksheetModel] {
        &self.worksheets
    }

    /// 返回可变 sheet 子流。
    #[must_use]
    pub fn worksheets_mut(&mut self) -> &mut Vec<Biff8WorksheetModel> {
        &mut self.worksheets
    }

    /// 对全部记录应用同一条 transform 链。
    ///
    /// # Errors
    ///
    /// transform 拒绝记录时返回原始错误。
    pub fn apply_transform(&mut self, transform: &mut dyn RecordTransform) -> Result<()> {
        transform_records(self.globals.records_mut(), transform)?;
        for sheet in &mut self.worksheets {
            transform_records(sheet.records_mut(), transform)?;
        }
        Ok(())
    }

    /// 两遍序列化 Workbook stream。
    ///
    /// 第一遍计算每个 sheet BOF 的绝对偏移并修补 BoundSheet；第二遍通过
    /// [`RecordSink`] 输出记录。未知记录不会被重编码。
    ///
    /// # Errors
    ///
    /// 记录过大、BoundSheet 数量不匹配或 sink 写入失败时返回错误。
    pub fn write_to(&self, sink: &mut dyn RecordSink) -> Result<()> {
        let mut globals = self.globals.records().to_vec();
        let bound_sheet_count = globals
            .iter()
            .filter(|record| record.sid() == BOUNDSHEET)
            .count();
        if bound_sheet_count != self.worksheets.len() {
            return Err(ExcelError::Xls(format!(
                "BOUNDSHEET count ({bound_sheet_count}) does not match sheet model count ({})",
                self.worksheets.len()
            )));
        }

        let mut next_sheet_offset = encoded_records_len(&globals)?;
        let mut sheet_offsets = Vec::with_capacity(self.worksheets.len());
        for sheet in &self.worksheets {
            sheet_offsets.push(
                u32::try_from(next_sheet_offset).map_err(|_| {
                    ExcelError::Xls("BIFF8 Workbook stream exceeds 4GiB".to_owned())
                })?,
            );
            next_sheet_offset = next_sheet_offset
                .checked_add(encoded_records_len(sheet.records())?)
                .ok_or_else(|| ExcelError::Xls("BIFF8 Workbook stream size overflow".to_owned()))?;
        }
        for (record, offset) in globals
            .iter_mut()
            .filter(|record| record.sid() == BOUNDSHEET)
            .zip(sheet_offsets)
        {
            if record.payload().len() < 4 {
                return Err(ExcelError::Xls("BOUNDSHEET record is too short".to_owned()));
            }
            record.payload_mut()[..4].copy_from_slice(&offset.to_le_bytes());
        }

        for record in &globals {
            sink.write_record(record)?;
        }
        for sheet in &self.worksheets {
            for record in sheet.records() {
                sink.write_record(record)?;
            }
        }
        Ok(())
    }

    /// 序列化为 Workbook stream 字节。
    ///
    /// # Errors
    ///
    /// 返回与 [`Self::write_to`] 相同的错误。
    pub fn to_workbook_stream(&self) -> Result<Vec<u8>> {
        let mut output = Vec::new();
        self.write_to(&mut output)?;
        Ok(output)
    }
}

fn transform_records(
    records: &mut Vec<Biff8Record>,
    transform: &mut dyn RecordTransform,
) -> Result<()> {
    let mut transformed = Vec::with_capacity(records.len());
    for record in std::mem::take(records) {
        if let Some(record) = transform.transform(record)? {
            transformed.push(record);
        }
    }
    *records = transformed;
    Ok(())
}

fn encoded_records_len(records: &[Biff8Record]) -> Result<usize> {
    records.iter().try_fold(0usize, |total, record| {
        if record.payload().len() > crate::biff8::encode::MAX_RECORD_DATA {
            return Err(ExcelError::Xls(format!(
                "BIFF record 0x{:04X} payload exceeds {} bytes",
                record.sid(),
                crate::biff8::encode::MAX_RECORD_DATA
            )));
        }
        total
            .checked_add(4 + record.payload().len())
            .ok_or_else(|| ExcelError::Xls("BIFF8 Workbook stream size overflow".to_owned()))
    })
}

fn top_level_substreams(records: &[Biff8Record]) -> Result<Vec<(usize, usize)>> {
    let mut spans = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    for (index, record) in records.iter().enumerate() {
        if record.sid() == BOF {
            if depth == 0 {
                start = Some(index);
            }
            depth = depth.saturating_add(1);
        } else if record.sid() == EOF {
            if depth == 0 {
                return Err(ExcelError::Xls(
                    "BIFF8 Workbook stream contains unmatched EOF".to_owned(),
                ));
            }
            depth -= 1;
            if depth == 0 {
                spans.push((start.take().expect("top-level BOF was recorded"), index));
            }
        }
    }
    if depth != 0 {
        return Err(ExcelError::Xls(
            "BIFF8 Workbook stream contains unterminated BOF substream".to_owned(),
        ));
    }
    Ok(spans)
}

fn is_worksheet_bof(payload: &[u8]) -> bool {
    payload.len() >= 4 && u16::from_le_bytes([payload[2], payload[3]]) == DT_WORKSHEET
}

fn decode_boundsheet_name(payload: &[u8]) -> Result<String> {
    if payload.len() < 8 {
        return Err(ExcelError::Xls("BOUNDSHEET record is too short".to_owned()));
    }
    let character_count = usize::from(payload[6]);
    let compressed = payload[7] & 0x01 == 0;
    let characters = &payload[8..];
    if compressed {
        if characters.len() < character_count {
            return Err(ExcelError::Xls(
                "BOUNDSHEET compressed name is truncated".to_owned(),
            ));
        }
        Ok(characters[..character_count]
            .iter()
            .map(|byte| char::from(*byte))
            .collect())
    } else {
        let byte_count = character_count
            .checked_mul(2)
            .ok_or_else(|| ExcelError::Xls("BOUNDSHEET name length overflow".to_owned()))?;
        if characters.len() < byte_count {
            return Err(ExcelError::Xls(
                "BOUNDSHEET Unicode name is truncated".to_owned(),
            ));
        }
        let units = characters[..byte_count]
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
            .collect::<Vec<_>>();
        String::from_utf16(&units)
            .map_err(|_| ExcelError::Xls("BOUNDSHEET name contains invalid UTF-16".to_owned()))
    }
}
