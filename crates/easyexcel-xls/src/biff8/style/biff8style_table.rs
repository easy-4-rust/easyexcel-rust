/// 对应 Java：无直接对应对象；Rust 架构扩展。 Workbook-global FONT / XF / number-format allocator。
#[derive(Debug, Clone, Default)]
pub struct Biff8StyleTable {
    /// Custom fonts beyond the five default Arial records.
    fonts: Vec<FontKey>,
    font_cache: HashMap<FontKey, u16>,
    /// Custom cell XF payloads (indices `XF_CUSTOM_BASE..`).
    xfs: Vec<[u8; 20]>,
    xf_cache: HashMap<XfKey, u16>,
    /// RGB colours allocated into the customizable palette (indices 8..).
    palette_rgb: Vec<(u8, u8, u8)>,
    /// Registered custom number formats `(ifmt, code)` in emission order.
    formats: Vec<(u16, String)>,
    /// Custom format code → ifmt lookup.
    format_lookup: HashMap<String, u16>,
}

impl Biff8StyleTable {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Resolves an XF index for `request`, preserving `base_xf` number format
    /// (`XF_GENERAL` / `XF_DATE` / `XF_DATETIME`).
    pub fn resolve_xf(&mut self, request: &Biff8StyleRequest, base_xf: u16) -> u16 {
        let base_ifmt = match base_xf {
            XF_DATE => 14,
            XF_DATETIME => 22,
            _ => 0,
        };
        let ifmt = self.resolve_ifmt(request.number_format.as_ref(), base_ifmt);
        if request.is_default() {
            return base_xf;
        }
        let font_index = self.ensure_font(request);
        let fill_fg_icv = self.resolve_color(request.fill_foreground_color, 0x40);
        let fill_bg_icv = self.resolve_color(request.fill_background_color, ICV_PATTERN_BG_DEFAULT);
        let key = XfKey {
            font_index,
            ifmt,
            halign: request
                .horizontal_alignment
                .map_or(0, Biff8HorizontalAlignment::code),
            valign: request
                .vertical_alignment
                .map_or(2, Biff8VerticalAlignment::code),
            wrap: request.wrap,
            fill_pattern: request.fill_pattern.map_or(0, Biff8FillPattern::code),
            fill_fg_icv,
            fill_bg_icv,
        };
        if let Some(existing) = self.xf_cache.get(&key) {
            return *existing;
        }
        let packed = pack_cell_xf(
            key.font_index,
            key.ifmt,
            key.halign,
            key.valign,
            key.wrap,
            key.fill_pattern,
            key.fill_fg_icv,
            key.fill_bg_icv,
        );
        // 语义敏感：自定义 XF 数量远小于 u16 上限，保留 as 以对齐 BIFF8 索引。
        #[allow(clippy::cast_possible_truncation)]
        let index = XF_CUSTOM_BASE + self.xfs.len() as u16;
        self.xfs.push(packed);
        self.xf_cache.insert(key, index);
        index
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 FONT records after the five defaults (emission order).
    #[must_use]
    pub fn custom_fonts(&self) -> Vec<Vec<u8>> {
        self.fonts
            .iter()
            .map(|font| {
                pack_font(
                    font.height_points,
                    font.bold,
                    font.italic,
                    font.strikeout,
                    font.color_icv,
                    &font.name,
                )
            })
            .collect()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Custom cell XF payloads in emission order.
    #[must_use]
    pub fn custom_xfs(&self) -> &[[u8; 20]] {
        &self.xfs
    }

    /// 解析数字格式为 BIFF8 ifmt：显式格式优先，其次 base（日期/时间），
    /// 最后 General(0)。自定义格式码从 164 起注册（同码复用）。
    fn resolve_ifmt(&mut self, format: Option<&Biff8NumberFormat>, base_ifmt: u16) -> u16 {
        let Some(format) = format else {
            return base_ifmt;
        };
        match format {
            Biff8NumberFormat::Builtin(index) => u16::from(*index),
            Biff8NumberFormat::Custom(code) => {
                if let Some(builtin) = builtin_format_id(code) {
                    return builtin;
                }
                if let Some(existing) = self.format_lookup.get(code.as_str()) {
                    return *existing;
                }
                // 语义敏感：自定义格式数量远小于 u16 上限，保留 as 以对齐 BIFF8 索引
                #[allow(clippy::cast_possible_truncation)]
                let ifmt = FORMAT_CUSTOM_BASE + self.formats.len() as u16;
                self.formats.push((ifmt, code.clone()));
                self.format_lookup.insert(code.clone(), ifmt);
                ifmt
            }
        }
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Registered custom FORMAT records in emission order.
    #[must_use]
    pub fn custom_formats(&self) -> &[(u16, String)] {
        &self.formats
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Whether a PALETTE record is required for custom RGB colours.
    #[must_use]
    pub fn needs_palette(&self) -> bool {
        !self.palette_rgb.is_empty()
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Custom RGB colours keyed by palette index starting at 8.
    #[must_use]
    pub fn palette_overrides(&self) -> &[(u8, u8, u8)] {
        &self.palette_rgb
    }

    fn ensure_font(&mut self, request: &Biff8StyleRequest) -> u16 {
        let key = FontKey {
            height_points: request.font_height_points.unwrap_or(10),
            bold: request.bold,
            italic: request.italic,
            strikeout: request.strikeout,
            color_icv: self.resolve_color(request.font_color, ICV_AUTO),
            name: request
                .font_name
                .clone()
                .unwrap_or_else(|| "Arial".to_owned()),
        };
        // Default Arial 10 / not bold / auto colour → built-in font 0.
        if key.height_points == 10
            && !key.bold
            && !key.italic
            && !key.strikeout
            && key.color_icv == ICV_AUTO
            && key.name == "Arial"
        {
            return 0;
        }
        if let Some(existing) = self.font_cache.get(&key) {
            return *existing;
        }
        // BIFF8 skips font index 4: slots 0..3 → indices 0..3, slot 4 → index 5, …
        let slot = 5 + self.fonts.len(); // 5th default is index 5; first custom → 6
        let index = font_index_for_slot(slot);
        self.fonts.push(key.clone());
        self.font_cache.insert(key, index);
        index
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Allocates or reuses a palette ICV for an RGB triple.
    // 语义敏感：BIFF8 调色板最多 56 色（索引 8..=63），usize->u16 不可能截断。
    #[allow(clippy::cast_possible_truncation)]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn alloc_rgb_icv(&mut self, rgb: u32) -> u16 {
        let r = ((rgb >> 16) & 0xFF) as u8;
        let g = ((rgb >> 8) & 0xFF) as u8;
        let b = (rgb & 0xFF) as u8;
        if let Some(pos) = self.palette_rgb.iter().position(|&c| c == (r, g, b)) {
            return (8 + pos) as u16;
        }
        if self.palette_rgb.len() >= 56 {
            // Fall back to nearest built-in when palette is full.
            return nearest_indexed(r, g, b);
        }
        let index = (8 + self.palette_rgb.len()) as u16;
        self.palette_rgb.push((r, g, b));
        index
    }

    fn resolve_color(&mut self, color: Option<Biff8Color>, default: u16) -> u16 {
        match color {
            None => default,
            Some(Biff8Color::Automatic) => ICV_AUTO,
            Some(Biff8Color::Indexed(index)) => u16::from(index),
            Some(Biff8Color::Rgb(rgb)) => self.alloc_rgb_icv(rgb),
        }
    }
}

