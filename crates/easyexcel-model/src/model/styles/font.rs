#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
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

