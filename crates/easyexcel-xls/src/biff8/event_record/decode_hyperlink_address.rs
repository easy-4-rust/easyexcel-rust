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

#[cfg(test)]
mod decode_hyperlink_address_tests {
    use super::*;

    /// 构造一个 UNC 路径类型的 HLINK record。
    /// options 包含 HLINK_URL | HLINK_UNC_PATH。
    fn build_unc_record(path: &str) -> Vec<u8> {
        let mut data = Vec::new();
        // CellRange: 8 bytes
        data.extend_from_slice(&[0u8; 8]);
        // standard moniker: 16 bytes (URL moniker)
        data.extend_from_slice(&URL_MONIKER);
        // stream version: 4 bytes
        data.extend_from_slice(&[0u8; 4]);
        // options: HLINK_URL(0x01) | HLINK_UNC_PATH(0x100) = 0x0101
        data.extend_from_slice(&0x0101u32.to_le_bytes());
        // UNC path as unicode string: count(4) + UTF-16LE bytes
        let units: Vec<u16> = path.encode_utf16().collect();
        data.extend_from_slice(&(units.len() as u32).to_le_bytes());
        for unit in &units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        data
    }

    /// 构造一个 URL 类型的 HLINK record（URL moniker + NUL 终止 UTF-16 地址）。
    ///
    /// 布局：CellRange(8) + std_moniker(16) + stream_version(4) + options(4) +
    ///       url_moniker_GUID(16) + byte_len(4) + NUL-terminated_UTF16LE
    fn build_url_record(url: &str) -> Vec<u8> {
        let mut data = Vec::new();
        // CellRange: 8 bytes
        data.extend_from_slice(&[0u8; 8]);
        // standard moniker: 16 bytes (任意值，代码不检查此处)
        data.extend_from_slice(&[0u8; 16]);
        // stream version: 4 bytes
        data.extend_from_slice(&[0u8; 4]);
        // options: HLINK_URL only (no UNC_PATH)
        data.extend_from_slice(&HLINK_URL.to_le_bytes());
        // URL moniker GUID（offset 32-47，代码在此处读取并比较）
        data.extend_from_slice(&URL_MONIKER);
        // URL moniker data: byte_len(4) + NUL-terminated UTF-16LE
        let units: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = units.len() * 2;
        data.extend_from_slice(&(byte_len as u32).to_le_bytes());
        for unit in &units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        data
    }

    /// 构造一个 document-place 类型的 HLINK record。
    fn build_place_record(place: &str) -> Vec<u8> {
        let mut data = Vec::new();
        // CellRange: 8 bytes
        data.extend_from_slice(&[0u8; 8]);
        // moniker: 16 bytes (任意非 URL moniker)
        data.extend_from_slice(&[0u8; 16]);
        // stream version: 4 bytes
        data.extend_from_slice(&[0u8; 4]);
        // options: HLINK_PLACE only
        data.extend_from_slice(&HLINK_PLACE.to_le_bytes());
        // place string: count(4) + UTF-16LE
        let units: Vec<u16> = place.encode_utf16().collect();
        data.extend_from_slice(&(units.len() as u32).to_le_bytes());
        for unit in &units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        data
    }

    /// UNC 路径应正确解码。
    #[test]
    fn unc_path_decoded() {
        let data = build_unc_record(r"\\server\share\file.xlsx");
        let addr = decode_hyperlink_address(&data).unwrap();
        assert_eq!(addr, r"\\server\share\file.xlsx");
    }

    /// URL moniker 应正确解码。
    #[test]
    fn url_moniker_decoded() {
        let data = build_url_record("https://example.com/path");
        let addr = decode_hyperlink_address(&data).unwrap();
        assert_eq!(addr, "https://example.com/path");
    }

    /// Document-place 应正确解码。
    #[test]
    fn place_decoded() {
        let data = build_place_record("Sheet1!A1");
        let addr = decode_hyperlink_address(&data).unwrap();
        assert_eq!(addr, "Sheet1!A1");
    }

    /// 数据太短时返回 None。
    #[test]
    fn too_short_returns_none() {
        assert_eq!(decode_hyperlink_address(&[0u8; 10]), None);
    }

    /// 空数据返回 None。
    #[test]
    fn empty_data_returns_none() {
        assert_eq!(decode_hyperlink_address(&[]), None);
    }

    /// 非 URL moniker 且非 PLACE 返回 None。
    #[test]
    fn unknown_moniker_returns_none() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 8]); // CellRange
        data.extend_from_slice(&[0xFF; 16]); // unknown moniker
        data.extend_from_slice(&[0u8; 4]); // stream version
        data.extend_from_slice(&HLINK_URL.to_le_bytes()); // options
        data.extend_from_slice(&[0u8; 4]); // byte_len=0
        assert_eq!(decode_hyperlink_address(&data), None);
    }

    /// HLINK_LABEL 标志位时跳过 label unicode string。
    #[test]
    fn label_flag_skipped() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 8]); // CellRange
        data.extend_from_slice(&[0u8; 16]); // standard moniker (任意)
        data.extend_from_slice(&[0u8; 4]); // stream version
        // options: HLINK_URL | HLINK_LABEL
        data.extend_from_slice(&(HLINK_URL | HLINK_LABEL).to_le_bytes());
        // label unicode string: count=3, UTF-16LE "abc"
        data.extend_from_slice(&3u32.to_le_bytes());
        for ch in ['a' as u16, 'b' as u16, 'c' as u16] {
            data.extend_from_slice(&ch.to_le_bytes());
        }
        // URL moniker GUID (offset 32+)
        data.extend_from_slice(&URL_MONIKER);
        // URL moniker data: byte_len + NUL-terminated UTF-16
        let url = "https://test.com";
        let units: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = units.len() * 2;
        data.extend_from_slice(&(byte_len as u32).to_le_bytes());
        for unit in &units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        let addr = decode_hyperlink_address(&data).unwrap();
        assert_eq!(addr, "https://test.com");
    }

    /// decode_nul_terminated_utf16 奇数长度返回 None。
    #[test]
    fn nul_terminated_odd_length_returns_none() {
        assert_eq!(decode_nul_terminated_utf16(&[0x41, 0x00, 0x00]), None);
    }

    /// decode_nul_terminated_utf16 空输入返回空字符串。
    #[test]
    fn nul_terminated_empty_returns_empty_string() {
        assert_eq!(decode_nul_terminated_utf16(&[]), Some(String::new()));
    }

    /// decode_nul_terminated_utf16 NUL 终止。
    #[test]
    fn nul_terminated_stops_at_nul() {
        let mut data = Vec::new();
        // "A\0B" → 只取 "A"
        data.extend_from_slice(&[0x41, 0x00]); // 'A'
        data.extend_from_slice(&[0x00, 0x00]); // NUL
        data.extend_from_slice(&[0x42, 0x00]); // 'B' (should be ignored)
        assert_eq!(decode_nul_terminated_utf16(&data), Some("A".to_owned()));
    }

    /// PLACE 优先于 URL 地址（当两者都存在时）。
    ///
    /// 布局：CellRange(8) + std_moniker(16) + stream_version(4) + options(4) +
    ///       url_moniker_GUID(16) + byte_len(4) + URL_data + PLACE_unicode_string
    #[test]
    fn place_overrides_url() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 8]); // CellRange
        data.extend_from_slice(&[0u8; 16]); // standard moniker (任意)
        data.extend_from_slice(&[0u8; 4]); // stream version
        // options: HLINK_URL | HLINK_PLACE (no UNC)
        data.extend_from_slice(&(HLINK_URL | HLINK_PLACE).to_le_bytes());
        // URL moniker GUID (offset 32-47)
        data.extend_from_slice(&URL_MONIKER);
        // URL moniker data (byte_len + NUL-terminated UTF-16)
        let url = "https://url.com";
        let units: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = units.len() * 2;
        data.extend_from_slice(&(byte_len as u32).to_le_bytes());
        for unit in &units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        // PLACE unicode string (紧跟在 URL 数据之后)
        let place = "Sheet2!B5";
        let place_units: Vec<u16> = place.encode_utf16().collect();
        data.extend_from_slice(&(place_units.len() as u32).to_le_bytes());
        for unit in &place_units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        let addr = decode_hyperlink_address(&data).unwrap();
        // PLACE 覆盖 URL
        assert_eq!(addr, "Sheet2!B5");
    }

    /// HLINK_TARGET_FRAME 标志位时跳过 target frame。
    #[test]
    fn target_frame_skipped() {
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 8]); // CellRange
        data.extend_from_slice(&[0u8; 16]); // standard moniker (任意)
        data.extend_from_slice(&[0u8; 4]); // stream version
        // options: HLINK_URL | HLINK_TARGET_FRAME
        data.extend_from_slice(&(HLINK_URL | HLINK_TARGET_FRAME).to_le_bytes());
        // target frame unicode string: count=5, "_blank"
        data.extend_from_slice(&5u32.to_le_bytes());
        for ch in ['_' as u16, 'b' as u16, 'l' as u16, 'a' as u16, 'n' as u16] {
            data.extend_from_slice(&ch.to_le_bytes());
        }
        // URL moniker GUID (offset 32+)
        data.extend_from_slice(&URL_MONIKER);
        // URL moniker data: byte_len + NUL-terminated UTF-16
        let url = "https://frame-test.com";
        let units: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = units.len() * 2;
        data.extend_from_slice(&(byte_len as u32).to_le_bytes());
        for unit in &units {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        let addr = decode_hyperlink_address(&data).unwrap();
        assert_eq!(addr, "https://frame-test.com");
    }
}
