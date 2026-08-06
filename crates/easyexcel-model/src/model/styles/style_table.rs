/// 对应 Java：无直接对应对象；Rust 架构扩展。 A deduplicating table of styles. Index 0 is always the default style.
#[derive(Debug, Clone)]
pub struct StyleTable {
    styles: Vec<CellStyle>,
    index: std::collections::HashMap<CellStyle, u32>,
}

impl Default for StyleTable {
    fn default() -> Self {
        let mut t = StyleTable {
            styles: Vec::new(),
            index: std::collections::HashMap::new(),
        };
        let _ = t.intern(CellStyle::default());
        t
    }
}

impl StyleTable {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Intern a style, returning its index (deduplicated).
    ///
    /// Excel permits far fewer than `u32::MAX` styles and allocating that many
    /// `CellStyle` values is not representable on supported targets, so the
    /// conversion is bounded by both the file format and address space.
    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn intern(&mut self, style: CellStyle) -> u32 {
        if let Some(&i) = self.index.get(&style) {
            return i;
        }
        let i = self.styles.len() as u32;
        self.index.insert(style.clone(), i);
        self.styles.push(style);
        i
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn get(&self, idx: u32) -> Option<&CellStyle> {
        self.styles.get(idx as usize)
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn len(&self) -> usize {
        self.styles.len()
    }

    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn iter(&self) -> impl Iterator<Item = &CellStyle> {
        self.styles.iter()
    }
}
