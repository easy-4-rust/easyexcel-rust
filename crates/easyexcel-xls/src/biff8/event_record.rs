//! BIFF8 事件读取所需的中立记录载荷解码器。

include!("event_record/biff8cell_header.rs");

include!("event_record/biff8number_record.rs");

include!("event_record/biff8label_sst_record.rs");

include!("event_record/biff8formula_cached_value.rs");

include!("event_record/biff8bof_type.rs");

include!("event_record/biff8formula_record.rs");

include!("event_record/biff8cell_range.rs");

include!("event_record/decode_hyperlink_address.rs");

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

/// 解码 BIFF8 `LABEL` 记录的坐标与内联字符串。
///
/// 对应 POI：`LabelRecord(RecordInputStream)`。字符标志位为 `0` 时，
/// 每个字节按 Unicode U+0000..U+00FF 映射；标志位为 `1` 时读取 UTF-16LE。
#[must_use]
pub fn decode_label_record(data: &[u8]) -> Option<(u32, usize, String)> {
    let (row, column) = decode_cell_position(data)?;
    let character_count = usize::from(u16::from_le_bytes(data.get(6..8)?.try_into().ok()?));
    let unicode_flag = *data.get(8)?;
    let raw = data.get(9..)?;
    let value = if unicode_flag & 0x01 == 0 {
        let bytes = raw.get(..character_count)?;
        bytes.iter().map(|byte| char::from(*byte)).collect()
    } else {
        let byte_count = character_count.checked_mul(2)?;
        let bytes = raw.get(..byte_count)?;
        let units = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    };
    Some((row, column, value))
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 解码 NOTE 记录的单元格坐标，并校验 BIFF8 NOTE 固定头长度。
#[must_use]
pub fn decode_note_record_position(data: &[u8]) -> Option<(u32, usize)> {
    if data.len() < 6 {
        return None;
    }
    decode_cell_position(data)
}

/// Decodes the shape id used to join NOTE with its preceding OBJ/TXO text.
///
/// 对应 Java：`NoteRecord#getShapeId()`。
#[must_use]
pub fn decode_note_shape_id(data: &[u8]) -> Option<u32> {
    Some(u32::from(u16::from_le_bytes(
        data.get(6..8)?.try_into().ok()?,
    )))
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── decode_cell_position ──────────────────────────────────────────

    #[test]
    fn cell_position_valid() {
        // row=3 (LE: 0x03,0x00), col=5 (LE: 0x05,0x00)
        let data = [0x03, 0x00, 0x05, 0x00];
        assert_eq!(decode_cell_position(&data), Some((3, 5)));
    }

    #[test]
    fn cell_position_too_short() {
        assert_eq!(decode_cell_position(&[0x01, 0x00, 0x02]), None);
    }

    // ── decode_label_record_position ──────────────────────────────────

    #[test]
    fn label_record_position_valid() {
        let mut data = vec![0x03, 0x00, 0x05, 0x00]; // row=3, col=5
        data.extend_from_slice(&[0u8; 4]); // padding to 8 bytes
        assert_eq!(decode_label_record_position(&data), Some((3, 5)));
    }

    #[test]
    fn label_record_position_too_short() {
        assert_eq!(decode_label_record_position(&[0u8; 7]), None);
    }

    // ── decode_label_record ───────────────────────────────────────────

    #[test]
    fn label_record_compressed() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x02, 0x00]); // row=2
        data.extend_from_slice(&[0x03, 0x00]); // col=3
        data.extend_from_slice(&[0u8; 2]); // xf
        data.extend_from_slice(&[0x03, 0x00]); // character_count=3
        data.push(0x00); // unicode_flag=0 (compressed)
        data.extend_from_slice(b"abc");
        let (row, col, value) = decode_label_record(&data).unwrap();
        assert_eq!(row, 2);
        assert_eq!(col, 3);
        assert_eq!(value, "abc");
    }

    #[test]
    fn label_record_wide() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0x00, 0x00]); // row=0
        data.extend_from_slice(&[0x00, 0x00]); // col=0
        data.extend_from_slice(&[0u8; 2]); // xf
        data.extend_from_slice(&[0x01, 0x00]); // character_count=1
        data.push(0x01); // unicode_flag=1 (wide)
        data.extend_from_slice(&[0x60, 0x4F]); // U+4F60 = '你'
        let (_, _, value) = decode_label_record(&data).unwrap();
        assert_eq!(value, "你");
    }

    #[test]
    fn label_record_too_short() {
        assert_eq!(decode_label_record(&[0u8; 5]), None);
    }

    // ── decode_note_record_position ───────────────────────────────────

    #[test]
    fn note_record_position_valid() {
        let data = [0x01, 0x00, 0x02, 0x00, 0xAA, 0xBB];
        assert_eq!(decode_note_record_position(&data), Some((1, 2)));
    }

    #[test]
    fn note_record_position_too_short() {
        assert_eq!(decode_note_record_position(&[0u8; 5]), None);
    }

    // ── decode_note_shape_id ──────────────────────────────────────────

    #[test]
    fn note_shape_id_valid() {
        // bytes 6..8 = shape_id LE
        let mut data = vec![0u8; 8];
        data[6] = 0x0A;
        data[7] = 0x00;
        assert_eq!(decode_note_shape_id(&data), Some(10));
    }

    #[test]
    fn note_shape_id_too_short() {
        assert_eq!(decode_note_shape_id(&[0u8; 7]), None);
    }

    // ── decode_obj_common_data ────────────────────────────────────────

    #[test]
    fn obj_common_data_with_ft_cmo() {
        let mut data = Vec::new();
        // subrecord ft=0x0015, cb=4
        data.extend_from_slice(&[0x15, 0x00, 0x04, 0x00]);
        // payload: object_type=0x0019(comment), object_id=42
        data.extend_from_slice(&[0x19, 0x00, 0x2A, 0x00]);
        let result = decode_obj_common_data(&data).unwrap();
        assert_eq!(result.object_type, 0x0019);
        assert_eq!(result.object_id, 42);
    }

    #[test]
    fn obj_common_data_skips_unknown_subrecords() {
        let mut data = Vec::new();
        // unknown subrecord ft=0x0001, cb=2
        data.extend_from_slice(&[0x01, 0x00, 0x02, 0x00]);
        data.extend_from_slice(&[0xAA, 0xBB]);
        // ftCmo
        data.extend_from_slice(&[0x15, 0x00, 0x04, 0x00]);
        data.extend_from_slice(&[0x01, 0x00, 0x05, 0x00]);
        let result = decode_obj_common_data(&data).unwrap();
        assert_eq!(result.object_type, 1);
        assert_eq!(result.object_id, 5);
    }

    #[test]
    fn obj_common_data_end_of_records_returns_none() {
        // ft=0x0000 → end marker
        let data = [0x00, 0x00, 0x00, 0x00];
        assert_eq!(decode_obj_common_data(&data), None);
    }

    #[test]
    fn obj_common_data_empty_returns_none() {
        assert_eq!(decode_obj_common_data(&[]), None);
    }

    // ── decode_cell_header ────────────────────────────────────────────

    #[test]
    fn cell_header_valid() {
        let data = [0x01, 0x00, 0x02, 0x00, 0x0F, 0x00]; // row=1, col=2, xf=15
        let header = decode_cell_header(&data).unwrap();
        assert_eq!(header.row, 1);
        assert_eq!(header.column, 2);
        assert_eq!(header.xf_index, 15);
    }

    #[test]
    fn cell_header_too_short() {
        assert_eq!(decode_cell_header(&[0u8; 5]), None);
    }

    // ── decode_bool_err_record ────────────────────────────────────────

    #[test]
    fn bool_err_record_true() {
        let mut data = vec![0u8; 8];
        data[6] = 0x01; // value=TRUE
        data[7] = 0x00; // type=boolean
        let result = decode_bool_err_record(&data).unwrap().unwrap();
        assert!(result.1);
    }

    #[test]
    fn bool_err_record_false() {
        let mut data = vec![0u8; 8];
        data[6] = 0x00; // value=FALSE
        data[7] = 0x00; // type=boolean
        let result = decode_bool_err_record(&data).unwrap().unwrap();
        assert!(!result.1);
    }

    #[test]
    fn bool_err_record_error_type() {
        let mut data = vec![0u8; 8];
        data[6] = 0x07; // error code
        data[7] = 0x01; // type=error
        let result = decode_bool_err_record(&data).unwrap();
        assert!(result.is_none()); // error branch → Some(None)
    }

    #[test]
    fn bool_err_record_too_short() {
        assert_eq!(decode_bool_err_record(&[0u8; 7]), None);
    }

    // ── decode_number_record ──────────────────────────────────────────

    #[test]
    fn number_record_valid() {
        let mut data = vec![0u8; 14];
        // row=0, col=0, xf=0
        let value: f64 = 42.5;
        data[6..14].copy_from_slice(&value.to_le_bytes());
        let record = decode_number_record(&data).unwrap();
        assert_eq!(record.value, 42.5);
    }

    #[test]
    fn number_record_too_short() {
        assert_eq!(decode_number_record(&[0u8; 13]), None);
    }

    // ── decode_label_sst_record ───────────────────────────────────────

    #[test]
    fn label_sst_record_valid() {
        let mut data = vec![0u8; 10];
        // row=0, col=0, xf=0
        data[6..10].copy_from_slice(&7u32.to_le_bytes()); // sst_index=7
        let record = decode_label_sst_record(&data).unwrap();
        assert_eq!(record.sst_index, 7);
    }

    #[test]
    fn label_sst_record_too_short() {
        assert_eq!(decode_label_sst_record(&[0u8; 9]), None);
    }

    // ── decode_formula_record ─────────────────────────────────────────

    #[test]
    fn formula_record_cached_number() {
        let mut data = vec![0u8; 14];
        let value: f64 = 3.14;
        data[6..14].copy_from_slice(&value.to_le_bytes());
        let record = decode_formula_record(&data).unwrap();
        match record.cached_value {
            Biff8FormulaCachedValue::Number(v) => assert!((v - 3.14).abs() < f64::EPSILON),
            other => panic!("expected Number, got {other:?}"),
        }
    }

    #[test]
    fn formula_record_cached_string() {
        let mut data = vec![0u8; 14];
        data[6..14].copy_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF]);
        let record = decode_formula_record(&data).unwrap();
        assert!(matches!(
            record.cached_value,
            Biff8FormulaCachedValue::String
        ));
    }

    #[test]
    fn formula_record_cached_boolean() {
        let mut data = vec![0u8; 14];
        data[6..14].copy_from_slice(&[0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF]);
        let record = decode_formula_record(&data).unwrap();
        assert!(matches!(
            record.cached_value,
            Biff8FormulaCachedValue::Boolean(true)
        ));
    }

    #[test]
    fn formula_record_cached_error() {
        let mut data = vec![0u8; 14];
        data[6..14].copy_from_slice(&[0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF]);
        let record = decode_formula_record(&data).unwrap();
        assert!(matches!(
            record.cached_value,
            Biff8FormulaCachedValue::Error
        ));
    }

    #[test]
    fn formula_record_cached_empty() {
        let mut data = vec![0u8; 14];
        data[6..14].copy_from_slice(&[0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF]);
        let record = decode_formula_record(&data).unwrap();
        assert!(matches!(
            record.cached_value,
            Biff8FormulaCachedValue::Empty
        ));
    }

    // ── decode_bof_type ───────────────────────────────────────────────

    #[test]
    fn bof_type_workbook() {
        let mut data = vec![0u8; 4];
        data[2..4].copy_from_slice(&5u16.to_le_bytes());
        assert!(matches!(
            decode_bof_type(&data),
            Some(Biff8BofType::Workbook)
        ));
    }

    #[test]
    fn bof_type_worksheet() {
        let mut data = vec![0u8; 4];
        data[2..4].copy_from_slice(&0x0010u16.to_le_bytes());
        assert!(matches!(
            decode_bof_type(&data),
            Some(Biff8BofType::Worksheet)
        ));
    }

    #[test]
    fn bof_type_other() {
        let mut data = vec![0u8; 4];
        data[2..4].copy_from_slice(&0x0040u16.to_le_bytes());
        assert!(matches!(
            decode_bof_type(&data),
            Some(Biff8BofType::Other(0x0040))
        ));
    }

    // ── decode_bound_sheet_record ─────────────────────────────────────

    #[test]
    fn bound_sheet_record_compressed() {
        let mut data = Vec::new();
        data.extend_from_slice(&100u32.to_le_bytes()); // bof_position=100
        data.extend_from_slice(&[0u8; 2]); // padding
        data.push(5); // character_count=5
        data.push(0x00); // flags: compressed
        data.extend_from_slice(b"Hello");
        let record = decode_bound_sheet_record(&data).unwrap();
        assert_eq!(record.name, "Hello");
        assert_eq!(record.bof_position, 100);
    }

    #[test]
    fn bound_sheet_record_wide() {
        let mut data = Vec::new();
        data.extend_from_slice(&50u32.to_le_bytes());
        data.extend_from_slice(&[0u8; 2]);
        data.push(2); // character_count=2
        data.push(0x01); // flags: wide
        data.extend_from_slice(&[0x60, 0x4F, 0x7D, 0x59]); // "你好"
        let record = decode_bound_sheet_record(&data).unwrap();
        assert_eq!(record.name, "你好");
        assert_eq!(record.bof_position, 50);
    }

    // ── decode_index_last_row ─────────────────────────────────────────

    #[test]
    fn index_last_row_valid() {
        let mut data = vec![0u8; 16];
        data[8..12].copy_from_slice(&100u32.to_le_bytes());
        assert_eq!(decode_index_last_row(&data), Some(100));
    }

    #[test]
    fn index_last_row_too_short() {
        assert_eq!(decode_index_last_row(&[0u8; 15]), None);
    }

    // ── decode_sst_unique_count ───────────────────────────────────────

    #[test]
    fn sst_unique_count_valid() {
        let mut data = vec![0u8; 8];
        data[4..8].copy_from_slice(&256u32.to_le_bytes());
        assert_eq!(decode_sst_unique_count(&data), Some(256));
    }

    // ── decode_cell_range ─────────────────────────────────────────────

    #[test]
    fn cell_range_valid() {
        let mut data = vec![0u8; 8];
        data[0..2].copy_from_slice(&0u16.to_le_bytes()); // first_row=0
        data[2..4].copy_from_slice(&5u16.to_le_bytes()); // last_row=5
        data[4..6].copy_from_slice(&1u16.to_le_bytes()); // first_col=1
        data[6..8].copy_from_slice(&3u16.to_le_bytes()); // last_col=3
        let range = decode_cell_range(&data).unwrap();
        assert_eq!(range.first_row, 0);
        assert_eq!(range.last_row, 5);
        assert_eq!(range.first_column, 1);
        assert_eq!(range.last_column, 3);
    }

    #[test]
    fn cell_range_too_short() {
        assert_eq!(decode_cell_range(&[0u8; 7]), None);
    }

    // ── decode_merge_ranges ───────────────────────────────────────────

    #[test]
    fn merge_ranges_valid() {
        let mut data = Vec::new();
        data.extend_from_slice(&2u16.to_le_bytes()); // count=2
        // range 1: rows 0..1, cols 0..1
        data.extend_from_slice(&[0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00]);
        // range 2: rows 5..5, cols 2..3
        data.extend_from_slice(&[0x05, 0x00, 0x05, 0x00, 0x02, 0x00, 0x03, 0x00]);
        let ranges = decode_merge_ranges(&data);
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].first_row, 0);
        assert_eq!(ranges[1].last_column, 3);
    }

    #[test]
    fn merge_ranges_empty() {
        assert!(decode_merge_ranges(&[]).is_empty());
    }

    #[test]
    fn merge_ranges_count_limits_output() {
        let mut data = Vec::new();
        data.extend_from_slice(&1u16.to_le_bytes()); // count=1
        data.extend_from_slice(&[0u8; 8]); // range 1
        data.extend_from_slice(&[0u8; 8]); // range 2 (should be ignored)
        let ranges = decode_merge_ranges(&data);
        assert_eq!(ranges.len(), 1);
    }

    // ── decode_text_object_fragment ───────────────────────────────────

    #[test]
    fn text_object_fragment_start() {
        // TxO 布局：bytes 0-1 保留，bytes 2-3 = object_id，bytes 12+ = text
        let mut data = vec![0u8; 12];
        data[2..4].copy_from_slice(&42u16.to_le_bytes()); // object_id=42
        data.extend_from_slice(b"hello\x00"); // text 从 offset 12 开始
        let result = decode_text_object_fragment(0x00B6, 0x00B6, 0x003C, &data).unwrap();
        match result {
            Biff8TextObjectFragment::Start { object_id, text } => {
                assert_eq!(object_id, 42);
                assert_eq!(text, Some("hello".to_owned()));
            }
            other => panic!("expected Start, got {other:?}"),
        }
    }

    #[test]
    fn text_object_fragment_continue() {
        let data = b"world\x00";
        let result = decode_text_object_fragment(0x003C, 0x00B6, 0x003C, data).unwrap();
        match result {
            Biff8TextObjectFragment::Continue(text) => assert_eq!(text, "world"),
            other => panic!("expected Continue, got {other:?}"),
        }
    }

    #[test]
    fn text_object_fragment_unrelated_sid() {
        let data = [0u8; 10];
        assert_eq!(
            decode_text_object_fragment(0x0099, 0x00B6, 0x003C, &data),
            None
        );
    }

    // ── decode_latin1_zero_terminated ─────────────────────────────────

    #[test]
    fn latin1_zero_terminated_basic() {
        assert_eq!(decode_latin1_zero_terminated(b"abc\x00def"), "abc");
    }

    #[test]
    fn latin1_zero_terminated_no_terminator() {
        assert_eq!(decode_latin1_zero_terminated(b"xyz"), "xyz");
    }

    #[test]
    fn latin1_zero_terminated_empty() {
        assert_eq!(decode_latin1_zero_terminated(b"\x00"), "");
    }
}
