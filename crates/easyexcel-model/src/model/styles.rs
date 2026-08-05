//! Cell styling: fonts, fills, borders, number formats, alignment, and the
//! deduplicating style table that the readers/writers index into.

/// Horizontal alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HAlign {
    #[default]
    General,
    Left,
    Center,
    Right,
    Fill,
    Justify,
    CenterContinuous,
    Distributed,
}

/// Vertical alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VAlign {
    Top,
    #[default]
    Bottom,
    Center,
    Justify,
    Distributed,
}

/// An ARGB color, e.g. `FFFF0000` for opaque red. `None` means "automatic".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Color(pub Option<u32>);

impl Color {
    pub const fn rgb(argb: u32) -> Self {
        Color(Some(argb))
    }
    pub const AUTO: Color = Color(None);
}

#[derive(Debug, Clone, PartialEq)]
pub struct Font {
    pub name: String,
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub color: Color,
}

impl Default for Font {
    fn default() -> Self {
        Font {
            name: "Calibri".to_string(),
            size: 11.0,
            bold: false,
            italic: false,
            underline: false,
            strike: false,
            color: Color::AUTO,
        }
    }
}

impl Eq for Font {}
impl std::hash::Hash for Font {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.size.to_bits().hash(state);
        self.bold.hash(state);
        self.italic.hash(state);
        self.underline.hash(state);
        self.strike.hash(state);
        self.color.hash(state);
    }
}

/// Fill pattern. We model the common solid fill plus "none"; other patterns are
/// preserved opaquely by index in the readers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Fill {
    pub pattern: FillPattern,
    pub fg: Color,
    pub bg: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FillPattern {
    #[default]
    None,
    Solid,
    Gray125,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Thin,
    Medium,
    Thick,
    Dashed,
    Dotted,
    Double,
    Hair,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BorderEdge {
    pub style: BorderStyle,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Borders {
    pub left: BorderEdge,
    pub right: BorderEdge,
    pub top: BorderEdge,
    pub bottom: BorderEdge,
}

/// A complete cell style. Equality/hash are used to deduplicate styles when
/// writing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct CellStyle {
    pub font: Font,
    pub fill: Fill,
    pub borders: Borders,
    /// The number-format code (e.g. `0.00`, `yyyy-mm-dd`). Empty == General.
    pub number_format: String,
    /// The original numFmtId if known (for round-trip of built-ins).
    pub number_format_id: Option<u16>,
    pub halign: HAlign,
    pub valign: VAlign,
    pub wrap_text: bool,
}

impl CellStyle {
    pub fn is_date(&self) -> bool {
        if let Some(id) = self.number_format_id
            && super::numfmt::is_date_format_id(id, Some(&self.number_format))
        {
            return true;
        }
        super::numfmt::is_date_format(&self.number_format)
    }
}

/// A deduplicating table of styles. Index 0 is always the default style.
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
        t.intern(CellStyle::default());
        t
    }
}

impl StyleTable {
    /// Intern a style, returning its index (deduplicated).
    pub fn intern(&mut self, style: CellStyle) -> u32 {
        if let Some(&i) = self.index.get(&style) {
            return i;
        }
        let i = self.styles.len() as u32;
        self.index.insert(style.clone(), i);
        self.styles.push(style);
        i
    }

    pub fn get(&self, idx: u32) -> Option<&CellStyle> {
        self.styles.get(idx as usize)
    }

    pub fn len(&self) -> usize {
        self.styles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.styles.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &CellStyle> {
        self.styles.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup() {
        let mut t = StyleTable::default();
        let mut s = CellStyle::default();
        s.font.bold = true;
        let a = t.intern(s.clone());
        let b = t.intern(s);
        assert_eq!(a, b);
        assert_eq!(t.len(), 2); // default + bold
    }

    #[test]
    fn date_detection() {
        let mut s = CellStyle {
            number_format: "yyyy-mm-dd".into(),
            ..Default::default()
        };
        assert!(s.is_date());
        s.number_format = "0.00".into();
        s.number_format_id = Some(2);
        assert!(!s.is_date());
    }
}
