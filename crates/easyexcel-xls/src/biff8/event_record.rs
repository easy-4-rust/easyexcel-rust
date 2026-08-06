//! BIFF8 事件读取所需的中立记录载荷解码器。

include!("event_record/biff8cell_header.rs");

include!("event_record/biff8number_record.rs");

include!("event_record/biff8label_sst_record.rs");

include!("event_record/biff8formula_cached_value.rs");

include!("event_record/biff8bof_type.rs");

include!("event_record/biff8formula_record.rs");

include!("event_record/biff8cell_range.rs");

include!("event_record/biff8bound_sheet_record.rs");

include!("event_record/biff8common_object_data.rs");

/// BIFF8 公共对象类型：批注。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub const BIFF8_OBJECT_TYPE_COMMENT: u16 = 0x0019;

include!("event_record/biff8text_object_fragment.rs");

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码单元格记录中的 `row|column` 坐标。
#[must_use]
pub fn decode_cell_position(data: &[u8]) -> Option<(u32, usize)> {
    if data.len() < 4 {
        return None;
    }
    Some((
        u32::from(u16::from_le_bytes([data[0], data[1]])),
        usize::from(u16::from_le_bytes([data[2], data[3]])),
    ))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 LABEL 记录的单元格坐标，并校验 BIFF8 LABEL 固定头长度。
#[must_use]
pub fn decode_label_record_position(data: &[u8]) -> Option<(u32, usize)> {
    if data.len() < 8 {
        return None;
    }
    decode_cell_position(data)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 NOTE 记录的单元格坐标，并校验 BIFF8 NOTE 固定头长度。
#[must_use]
pub fn decode_note_record_position(data: &[u8]) -> Option<(u32, usize)> {
    if data.len() < 6 {
        return None;
    }
    decode_cell_position(data)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 OBJ 记录的 `ftCmo`（Common Object Data）子记录。
///
/// OBJ 由一组 `ft|cb|payload` 子记录组成；本函数逐段校验长度，并从
/// `ftCmo(0x0015)` 载荷读取对象类型与对象编号。
#[must_use]
pub fn decode_obj_common_data(data: &[u8]) -> Option<Biff8CommonObjectData> {
    let mut offset = 0usize;
    while offset.checked_add(4)? <= data.len() {
        let subrecord_type = u16::from_le_bytes(data.get(offset..offset + 2)?.try_into().ok()?);
        let payload_len = usize::from(u16::from_le_bytes(
            data.get(offset + 2..offset + 4)?.try_into().ok()?,
        ));
        let payload_start = offset.checked_add(4)?;
        let payload_end = payload_start.checked_add(payload_len)?;
        let payload = data.get(payload_start..payload_end)?;
        if subrecord_type == 0x0015 {
            let object_type = u16::from_le_bytes(payload.get(0..2)?.try_into().ok()?);
            let object_id = u32::from(u16::from_le_bytes(payload.get(2..4)?.try_into().ok()?));
            return Some(Biff8CommonObjectData {
                object_type,
                object_id,
            });
        }
        if subrecord_type == 0 {
            return None;
        }
        offset = payload_end;
    }
    None
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码带有 `row|column|xf` 的单元格公共头。
#[must_use]
pub fn decode_cell_header(data: &[u8]) -> Option<Biff8CellHeader> {
    if data.len() < 6 {
        return None;
    }
    let (row, column) = decode_cell_position(data)?;
    Some(Biff8CellHeader {
        row,
        column,
        xf_index: u16::from_le_bytes([data[4], data[5]]),
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 BOOLERR 的布尔分支；错误分支返回 `Some(None)`。
#[must_use]
pub fn decode_bool_err_record(data: &[u8]) -> Option<Option<(Biff8CellHeader, bool)>> {
    if data.len() < 8 {
        return None;
    }
    if data[7] != 0 {
        return Some(None);
    }
    decode_cell_header(data).map(|header| Some((header, data[6] != 0)))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 NUMBER 记录。
#[must_use]
pub fn decode_number_record(data: &[u8]) -> Option<Biff8NumberRecord> {
    let header = decode_cell_header(data)?;
    let bytes: [u8; 8] = data.get(6..14)?.try_into().ok()?;
    Some(Biff8NumberRecord {
        header,
        value: f64::from_le_bytes(bytes),
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 LABELSST 记录。
#[must_use]
pub fn decode_label_sst_record(data: &[u8]) -> Option<Biff8LabelSstRecord> {
    let header = decode_cell_header(data)?;
    let index = u32::from_le_bytes(data.get(6..10)?.try_into().ok()?);
    Some(Biff8LabelSstRecord {
        header,
        sst_index: usize::try_from(index).ok()?,
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 FORMULA 记录的公共头和缓存结果。
#[must_use]
pub fn decode_formula_record(data: &[u8]) -> Option<Biff8FormulaRecord> {
    let header = decode_cell_header(data)?;
    let result: [u8; 8] = data.get(6..14)?.try_into().ok()?;
    let cached_value = if result[6] == 0xFF && result[7] == 0xFF {
        match result[0] {
            0x00 => Biff8FormulaCachedValue::String,
            0x01 => Biff8FormulaCachedValue::Boolean(result[2] != 0),
            0x02 => Biff8FormulaCachedValue::Error,
            _ => Biff8FormulaCachedValue::Empty,
        }
    } else {
        Biff8FormulaCachedValue::Number(f64::from_le_bytes(result))
    };
    Some(Biff8FormulaRecord {
        header,
        cached_value,
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 BOF 记录的语义化子流类型。
#[must_use]
pub fn decode_bof_type(data: &[u8]) -> Option<Biff8BofType> {
    let code = data
        .get(2..4)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u16::from_le_bytes)?;
    Some(match code {
        0x0005 => Biff8BofType::Workbook,
        0x0010 => Biff8BofType::Worksheet,
        other => Biff8BofType::Other(other),
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 BOUNDSHEET 记录。
#[must_use]
pub fn decode_bound_sheet_record(data: &[u8]) -> Option<Biff8BoundSheetRecord> {
    let bof_position = u32::from_le_bytes(data.get(0..4)?.try_into().ok()?);
    let character_count = usize::from(*data.get(6)?);
    let is_utf16 = *data.get(7)? & 0x01 != 0;
    let raw = data.get(8..)?;
    let name = if is_utf16 {
        let byte_count = character_count.checked_mul(2)?;
        let body = raw.get(..byte_count)?;
        let units = body
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    } else {
        raw.get(..character_count)?
            .iter()
            .map(|byte| char::from(*byte))
            .collect()
    };
    Some(Biff8BoundSheetRecord { name, bof_position })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 INDEX 记录的 `lastRowAdd1`。
#[must_use]
pub fn decode_index_last_row(data: &[u8]) -> Option<u32> {
    if data.len() < 16 {
        return None;
    }
    data.get(8..12)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u32::from_le_bytes)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 SST 记录的唯一字符串数量。
#[must_use]
pub fn decode_sst_unique_count(data: &[u8]) -> Option<u32> {
    data.get(4..8)
        .and_then(|bytes| bytes.try_into().ok())
        .map(u32::from_le_bytes)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码一个 8 字节单元格区域。
#[must_use]
pub fn decode_cell_range(data: &[u8]) -> Option<Biff8CellRange> {
    if data.len() < 8 {
        return None;
    }
    Some(Biff8CellRange {
        first_row: u32::from(u16::from_le_bytes([data[0], data[1]])),
        last_row: u32::from(u16::from_le_bytes([data[2], data[3]])),
        first_column: usize::from(u16::from_le_bytes([data[4], data[5]])),
        last_column: usize::from(u16::from_le_bytes([data[6], data[7]])),
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 MERGECELLS 记录中的完整区域列表。
#[must_use]
pub fn decode_merge_ranges(data: &[u8]) -> Vec<Biff8CellRange> {
    if data.len() < 2 {
        return Vec::new();
    }
    let count = usize::from(u16::from_le_bytes([data[0], data[1]]));
    data[2..]
        .chunks_exact(8)
        .take(count)
        .filter_map(decode_cell_range)
        .collect()
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 `TxO` 起始记录或 CONTINUE 文本片段。
#[must_use]
pub fn decode_text_object_fragment(
    record_sid: u16,
    text_object_sid: u16,
    continue_sid: u16,
    data: &[u8],
) -> Option<Biff8TextObjectFragment> {
    if record_sid == text_object_sid {
        let object_id = u32::from(u16::from_le_bytes(data.get(2..4)?.try_into().ok()?));
        let text = data
            .get(12..)
            .map(decode_latin1_zero_terminated)
            .filter(|text| !text.is_empty());
        return Some(Biff8TextObjectFragment::Start { object_id, text });
    }
    if record_sid == continue_sid {
        if data.len() < 2 {
            return None;
        }
        let text = decode_latin1_zero_terminated(data);
        return (!text.is_empty()).then_some(Biff8TextObjectFragment::Continue(text));
    }
    None
}

fn decode_latin1_zero_terminated(data: &[u8]) -> String {
    data.iter()
        .take_while(|&&byte| byte != 0)
        .map(|&byte| char::from(byte))
        .collect()
}
