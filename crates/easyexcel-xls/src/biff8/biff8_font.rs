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
    pub const fn height_twips(&self) -> u16 { self.height_twips }

    /// 返回是否使用斜体。
    #[must_use]
    pub const fn italic(&self) -> bool { self.italic }

    /// 返回是否使用删除线。
    #[must_use]
    pub const fn strikeout(&self) -> bool { self.strikeout }

    /// 返回是否使用粗体。
    #[must_use]
    pub const fn bold(&self) -> bool { self.bold }

    /// 返回 BIFF 字符集编号。
    #[must_use]
    pub const fn charset(&self) -> u8 { self.charset }

    /// 返回字体名称。
    #[must_use]
    pub fn name(&self) -> &str { &self.name }

    /// 返回有效的 BIFF 调色板索引。
    #[must_use]
    pub const fn color_index(&self) -> Option<u8> { self.color_index }

    /// 返回 BIFF 上下标原始值：0 普通、1 上标、2 下标。
    #[must_use]
    pub const fn script(&self) -> u16 { self.script }

    /// 返回 BIFF 下划线原始值。
    #[must_use]
    pub const fn underline(&self) -> u8 { self.underline }
}
