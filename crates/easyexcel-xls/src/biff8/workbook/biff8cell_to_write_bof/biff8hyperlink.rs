/// BIFF8 URL hyperlink attached to one cell.
///
/// 对应 Java：`org.apache.poi.hssf.record.HyperlinkRecord`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8Hyperlink {
    first_row: u16,
    last_row: u16,
    first_col: u8,
    last_col: u8,
    url: String,
    label: String,
    kind: Biff8HyperlinkKind,
}

impl Biff8Hyperlink {
    const STD_MONIKER: [u8; 16] = [
        0xD0, 0xC9, 0xEA, 0x79, 0xF9, 0xBA, 0xCE, 0x11, 0x8C, 0x82, 0x00, 0xAA, 0x00, 0x4B,
        0xA9, 0x0B,
    ];
    const URL_MONIKER: [u8; 16] = [
        0xE0, 0xC9, 0xEA, 0x79, 0xF9, 0xBA, 0xCE, 0x11, 0x8C, 0x82, 0x00, 0xAA, 0x00, 0x4B,
        0xA9, 0x0B,
    ];
    const FILE_MONIKER: [u8; 16] = [
        0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x46,
    ];
    const URL_TAIL: [u8; 24] = [
        0x79, 0x58, 0x81, 0xF4, 0x3B, 0x1D, 0x7F, 0x48, 0xAF, 0x2C, 0x82, 0x5D, 0xC4, 0x85,
        0x27, 0x63, 0x00, 0x00, 0x00, 0x00, 0xA5, 0xAB, 0x00, 0x00,
    ];
    const FILE_TAIL: [u8; 28] = [
        0xFF, 0xFF, 0xAD, 0xDE, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0,
    ];

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new_range(
        first_row: u16,
        last_row: u16,
        first_col: u8,
        last_col: u8,
        url: String,
        label: String,
        kind: Biff8HyperlinkKind,
    ) -> Result<Self> {
        if url.contains('\0') || label.contains('\0') {
            return Err(ExcelError::Xls(
                "BIFF8 hyperlink URL and label must not contain NUL".to_owned(),
            ));
        }
        if last_row < first_row || last_col < first_col {
            return Err(ExcelError::Xls(
                "BIFF8 hyperlink range end must not precede its start".to_owned(),
            ));
        }
        let hyperlink = Self {
            first_row,
            last_row,
            first_col,
            last_col,
            url,
            label,
            kind,
        };
        if hyperlink.encode_record_data().len() > MAX_RECORD_DATA {
            return Err(ExcelError::Xls(format!(
                "BIFF8 hyperlink payload exceeds {MAX_RECORD_DATA} bytes"
            )));
        }
        Ok(hyperlink)
    }

    /// Encodes the HLINK payload used by Apache POI's `newUrlLink` path.
    pub(crate) fn encode_record_data(&self) -> Vec<u8> {
        let label = nul_terminated_utf16(&self.label);
        let mut data = Vec::with_capacity(80 + (label.len() + self.url.len()) * 2);
        data.extend_from_slice(&self.first_row.to_le_bytes());
        data.extend_from_slice(&self.last_row.to_le_bytes());
        data.extend_from_slice(&u16::from(self.first_col).to_le_bytes());
        data.extend_from_slice(&u16::from(self.last_col).to_le_bytes());
        data.extend_from_slice(&Self::STD_MONIKER);
        data.extend_from_slice(&2u32.to_le_bytes());
        let flags = match self.kind {
            Biff8HyperlinkKind::Url | Biff8HyperlinkKind::Email => 0x17u32,
            Biff8HyperlinkKind::Document => 0x1Cu32,
            Biff8HyperlinkKind::File => 0x15u32,
        };
        data.extend_from_slice(&flags.to_le_bytes());
        data.extend_from_slice(&u32::try_from(label.len()).unwrap_or(u32::MAX).to_le_bytes());
        append_utf16(&mut data, &label);
        match self.kind {
            Biff8HyperlinkKind::Url | Biff8HyperlinkKind::Email => {
                let url = nul_terminated_utf16(&self.url);
                data.extend_from_slice(&Self::URL_MONIKER);
                let url_bytes = url.len().saturating_mul(2).saturating_add(Self::URL_TAIL.len());
                data.extend_from_slice(
                    &u32::try_from(url_bytes).unwrap_or(u32::MAX).to_le_bytes(),
                );
                append_utf16(&mut data, &url);
                data.extend_from_slice(&Self::URL_TAIL);
            }
            Biff8HyperlinkKind::Document => {
                let location = nul_terminated_utf16(&self.url);
                data.extend_from_slice(
                    &u32::try_from(location.len()).unwrap_or(u32::MAX).to_le_bytes(),
                );
                append_utf16(&mut data, &location);
            }
            Biff8HyperlinkKind::File => {
                let path = self.url.as_bytes();
                data.extend_from_slice(&Self::FILE_MONIKER);
                data.extend_from_slice(&0u16.to_le_bytes());
                data.extend_from_slice(
                    &u32::try_from(path.len().saturating_add(1))
                        .unwrap_or(u32::MAX)
                        .to_le_bytes(),
                );
                data.extend_from_slice(path);
                data.push(0);
                data.extend_from_slice(&Self::FILE_TAIL);
            }
        }
        data
    }
}

fn nul_terminated_utf16(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn append_utf16(output: &mut Vec<u8>, value: &[u16]) {
    for unit in value {
        output.extend_from_slice(&unit.to_le_bytes());
    }
}
