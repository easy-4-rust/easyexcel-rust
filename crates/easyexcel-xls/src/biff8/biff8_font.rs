//! BIFF8 FONT 记录的中立表示。

/// 从 BIFF8 `FONT` 记录解码出的字体属性。
///
/// 该类型只保存格式层事实，不依赖 `easyexcel` 门面的 `WriteFont`，由上层适配器
/// 决定如何映射为公开 API 元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Biff8Font {
    height_twips: u16,
    italic: bool,
    strikeout: bool,
    bold: bool,
    charset: u8,
    name: String,
    color_index: Option<u8>,
    script: u16,
    underline: u8,
}

impl Biff8Font {
    /// 从一条 BIFF8 `FONT` record payload 解码字体；截断记录返回 `None`。
    #[must_use]
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 16 {
            return None;
        }
        let height_twips = u16::from_le_bytes([data[0], data[1]]);
        let options = u16::from_le_bytes([data[2], data[3]]);
        let raw_color_index = u16::from_le_bytes([data[4], data[5]]);
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
        let color_index = u8::try_from(raw_color_index)
            .ok()
            .filter(|index| *index <= 64);
        Some(Self {
            height_twips,
            italic: options & 0x0002 != 0,
            strikeout: options & 0x0008 != 0,
            bold: weight >= 700,
            charset: data[12],
            name,
            color_index,
            script,
            underline: data[10],
        })
    }

    /// 返回字体高度，单位为二十分之一磅。
    #[must_use]
    pub const fn height_twips(&self) -> u16 {
        self.height_twips
    }

    /// 返回是否使用斜体。
    #[must_use]
    pub const fn italic(&self) -> bool {
        self.italic
    }

    /// 返回是否使用删除线。
    #[must_use]
    pub const fn strikeout(&self) -> bool {
        self.strikeout
    }

    /// 返回是否使用粗体。
    #[must_use]
    pub const fn bold(&self) -> bool {
        self.bold
    }

    /// 返回 BIFF 字符集编号。
    #[must_use]
    pub const fn charset(&self) -> u8 {
        self.charset
    }

    /// 返回字体名称。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 返回有效的 BIFF 调色板索引。
    #[must_use]
    pub const fn color_index(&self) -> Option<u8> {
        self.color_index
    }

    /// 返回 BIFF 上下标原始值：0 普通、1 上标、2 下标。
    #[must_use]
    pub const fn script(&self) -> u16 {
        self.script
    }

    /// 返回 BIFF 下划线原始值。
    #[must_use]
    pub const fn underline(&self) -> u8 {
        self.underline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal BIFF8 FONT record payload with a compressed (Latin1) name.
    fn build_font_record(
        height_twips: u16,
        options: u16,
        color_index: u16,
        weight: u16,
        script: u16,
        underline: u8,
        charset: u8,
        name: &str,
    ) -> Vec<u8> {
        let mut data = Vec::with_capacity(16 + name.len());
        data.extend_from_slice(&height_twips.to_le_bytes());
        data.extend_from_slice(&options.to_le_bytes());
        data.extend_from_slice(&color_index.to_le_bytes());
        data.extend_from_slice(&weight.to_le_bytes());
        data.extend_from_slice(&script.to_le_bytes());
        data.push(underline);
        data.push(0); // reserved
        data.push(charset);
        data.push(0); // reserved
        data.push(name.len() as u8);
        data.push(0x00); // grbit: compressed
        data.extend(name.bytes());
        data
    }

    /// Build a BIFF8 FONT record payload with a wide (UTF-16) name.
    fn build_font_record_wide(
        height_twips: u16,
        options: u16,
        color_index: u16,
        weight: u16,
        script: u16,
        underline: u8,
        charset: u8,
        name: &str,
    ) -> Vec<u8> {
        let utf16: Vec<u16> = name.encode_utf16().collect();
        let mut data = Vec::with_capacity(16 + utf16.len() * 2);
        data.extend_from_slice(&height_twips.to_le_bytes());
        data.extend_from_slice(&options.to_le_bytes());
        data.extend_from_slice(&color_index.to_le_bytes());
        data.extend_from_slice(&weight.to_le_bytes());
        data.extend_from_slice(&script.to_le_bytes());
        data.push(underline);
        data.push(0); // reserved
        data.push(charset);
        data.push(0); // reserved
        data.push(utf16.len() as u8);
        data.push(0x01); // grbit: wide
        for unit in &utf16 {
            data.extend_from_slice(&unit.to_le_bytes());
        }
        data
    }

    #[test]
    fn decode_basic_font_compressed_name() {
        let data = build_font_record(200, 0, 0, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.height_twips(), 200);
        assert!(!font.italic());
        assert!(!font.strikeout());
        assert!(!font.bold());
        assert_eq!(font.charset(), 0);
        assert_eq!(font.name(), "Arial");
        assert_eq!(font.color_index(), Some(0));
        assert_eq!(font.script(), 0);
        assert_eq!(font.underline(), 0);
    }

    #[test]
    fn decode_font_with_italic_flag() {
        let data = build_font_record(240, 0x0002, 0, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert!(font.italic());
    }

    #[test]
    fn decode_font_with_strikeout_flag() {
        let data = build_font_record(240, 0x0008, 0, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert!(font.strikeout());
    }

    #[test]
    fn decode_font_with_italic_and_strikeout() {
        let data = build_font_record(240, 0x000A, 0, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert!(font.italic());
        assert!(font.strikeout());
    }

    #[test]
    fn decode_bold_font_weight_threshold() {
        // Weight >= 700 means bold
        let data = build_font_record(240, 0, 0, 700, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert!(font.bold());

        let data = build_font_record(240, 0, 0, 699, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert!(!font.bold());
    }

    #[test]
    fn decode_font_color_index_valid() {
        let data = build_font_record(240, 0, 64, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.color_index(), Some(64));
    }

    #[test]
    fn decode_font_color_index_out_of_range() {
        let data = build_font_record(240, 0, 65, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.color_index(), None);
    }

    #[test]
    fn decode_font_color_index_max_u16() {
        let data = build_font_record(240, 0, 0xFFFF, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.color_index(), None);
    }

    #[test]
    fn decode_font_with_script_superscript() {
        let data = build_font_record(240, 0, 0, 400, 1, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.script(), 1);
    }

    #[test]
    fn decode_font_with_script_subscript() {
        let data = build_font_record(240, 0, 0, 400, 2, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.script(), 2);
    }

    #[test]
    fn decode_font_with_underline_single() {
        let data = build_font_record(240, 0, 0, 400, 0, 1, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.underline(), 1);
    }

    #[test]
    fn decode_font_with_charset() {
        let data = build_font_record(240, 0, 0, 400, 0, 0, 0x80, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.charset(), 0x80);
    }

    #[test]
    fn decode_font_wide_name() {
        let data = build_font_record_wide(240, 0, 0, 400, 0, 0, 0, "Test");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.name(), "Test");
    }

    #[test]
    fn decode_font_wide_unicode_name() {
        let data = build_font_record_wide(240, 0, 0, 400, 0, 0, 0, "你好");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.name(), "你好");
    }

    #[test]
    fn decode_font_returns_none_for_truncated_data() {
        // Less than 16 bytes
        let data = vec![0u8; 15];
        assert!(Biff8Font::decode(&data).is_none());
    }

    #[test]
    fn decode_font_returns_none_for_empty_data() {
        assert!(Biff8Font::decode(&[]).is_none());
    }

    #[test]
    fn decode_font_returns_none_when_name_exceeds_data() {
        // Name length claims 10 chars but data is too short
        let mut data = build_font_record(240, 0, 0, 400, 0, 0, 0, "AB");
        // Corrupt the name length byte to claim more chars than available
        data[14] = 100;
        assert!(Biff8Font::decode(&data).is_none());
    }

    #[test]
    fn decode_font_wide_name_returns_none_when_data_truncated() {
        let mut data = build_font_record_wide(240, 0, 0, 400, 0, 0, 0, "AB");
        // Corrupt the name length to claim more wide chars than available
        data[14] = 100;
        assert!(Biff8Font::decode(&data).is_none());
    }

    #[test]
    fn decode_font_all_properties_combined() {
        let data = build_font_record(360, 0x000A, 32, 700, 1, 2, 1, "Courier New");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.height_twips(), 360);
        assert!(font.italic());
        assert!(font.strikeout());
        assert!(font.bold());
        assert_eq!(font.charset(), 1);
        assert_eq!(font.name(), "Courier New");
        assert_eq!(font.color_index(), Some(32));
        assert_eq!(font.script(), 1);
        assert_eq!(font.underline(), 2);
    }

    #[test]
    fn decode_font_zero_height() {
        let data = build_font_record(0, 0, 0, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.height_twips(), 0);
    }

    #[test]
    fn decode_font_max_height() {
        let data = build_font_record(u16::MAX, 0, 0, 400, 0, 0, 0, "Arial");
        let font = Biff8Font::decode(&data).expect("should decode");
        assert_eq!(font.height_twips(), u16::MAX);
    }
}
