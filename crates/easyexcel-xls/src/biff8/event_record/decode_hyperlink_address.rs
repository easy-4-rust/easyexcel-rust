const HLINK_URL: u32 = 0x01;
const HLINK_LABEL: u32 = 0x14;
const HLINK_PLACE: u32 = 0x08;
const HLINK_TARGET_FRAME: u32 = 0x80;
const HLINK_UNC_PATH: u32 = 0x100;

const URL_MONIKER: [u8; 16] = [
    0xE0, 0xC9, 0xEA, 0x79, 0xF9, 0xBA, 0xCE, 0x11, 0x8C, 0x82, 0x00, 0xAA, 0x00, 0x4B,
    0xA9, 0x0B,
];

/// Decodes the address carried by a BIFF8 HLINK record.
///
/// 对应 Java：`HyperlinkRecord#getAddress()`。当前覆盖 Java `EasyExcel` 会生成
/// 的 URL、UNC 和 document-place 链接；损坏或未知 moniker 返回 `None`。
#[must_use]
pub fn decode_hyperlink_address(data: &[u8]) -> Option<String> {
    // CellRange(8) + standard moniker(16) + stream version(4) + options(4).
    let mut offset = 28usize;
    let options = read_u32(data, &mut offset)?;

    if options & HLINK_LABEL != 0 {
        skip_unicode_string(data, &mut offset)?;
    }
    if options & HLINK_TARGET_FRAME != 0 {
        skip_unicode_string(data, &mut offset)?;
    }

    let mut address = None;
    if options & HLINK_URL != 0 && options & HLINK_UNC_PATH != 0 {
        address = read_unicode_string(data, &mut offset);
    } else if options & HLINK_URL != 0 {
        let moniker = data.get(offset..offset.checked_add(16)?)?;
        offset = offset.checked_add(16)?;
        if moniker == URL_MONIKER {
            let byte_len = usize::try_from(read_u32(data, &mut offset)?).ok()?;
            let bytes = data.get(offset..offset.checked_add(byte_len)?)?;
            address = decode_nul_terminated_utf16(bytes);
            offset = offset.checked_add(byte_len)?;
        } else {
            // File monikers have a different variable-length structure. Do not
            // guess or expose a truncated path as a valid address.
            return None;
        }
    }

    if options & HLINK_PLACE != 0 {
        return read_unicode_string(data, &mut offset).or(address);
    }
    address
}

fn read_u32(data: &[u8], offset: &mut usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let value = u32::from_le_bytes(data.get(*offset..end)?.try_into().ok()?);
    *offset = end;
    Some(value)
}

fn skip_unicode_string(data: &[u8], offset: &mut usize) -> Option<()> {
    let units = usize::try_from(read_u32(data, offset)?).ok()?;
    let bytes = units.checked_mul(2)?;
    *offset = offset.checked_add(bytes)?;
    data.get(..*offset).map(|_| ())
}

fn read_unicode_string(data: &[u8], offset: &mut usize) -> Option<String> {
    let units = usize::try_from(read_u32(data, offset)?).ok()?;
    let bytes = units.checked_mul(2)?;
    let end = offset.checked_add(bytes)?;
    let value = decode_nul_terminated_utf16(data.get(*offset..end)?)?;
    *offset = end;
    Some(value)
}

fn decode_nul_terminated_utf16(bytes: &[u8]) -> Option<String> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .take_while(|unit| *unit != 0)
        .collect::<Vec<_>>();
    String::from_utf16(&units).ok()
}
