/// 对应 Java：无直接对应对象；Rust 架构扩展。 Resolved write-style inputs used when allocating an XF index.
// 语义敏感：bold/italic/strikeout/wrap 与 Java `WriteCellStyle`/`WriteFont`
// 布尔字段一一对应，合并会破坏 1:1 可追溯性，故豁免 struct_excessive_bools。
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Default)]
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub struct Biff8StyleRequest {
    /// Bold font.
    pub bold: bool,
    /// Italic font.
    pub italic: bool,
    /// Strike-through font.
    pub strikeout: bool,
    /// Font height in points (`None` → 10pt Arial default).
    pub font_height_points: Option<u16>,
    /// Font family name (`None` → `"Arial"`).
    pub font_name: Option<String>,
    /// Semantic font colour (`None` → automatic).
    pub font_color: Option<Biff8Color>,
    /// Horizontal alignment (`None` → general).
    pub horizontal_alignment: Option<Biff8HorizontalAlignment>,
    /// Vertical alignment (`None` → bottom).
    pub vertical_alignment: Option<Biff8VerticalAlignment>,
    /// Wrap text.
    pub wrap: bool,
    /// Semantic fill pattern (`None` / [`Biff8FillPattern::None`] → no fill).
    pub fill_pattern: Option<Biff8FillPattern>,
    /// Fill foreground colour.
    pub fill_foreground_color: Option<Biff8Color>,
    /// Fill background colour.
    pub fill_background_color: Option<Biff8Color>,
    /// Number format: built-in index or custom code.
    pub number_format: Option<Biff8NumberFormat>,
}

impl Biff8StyleRequest {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 Returns `true` when this request would produce `XF_GENERAL` with default font.
    #[must_use]
    pub fn is_default(&self) -> bool {
        !self.bold
            && !self.italic
            && !self.strikeout
            && self.font_height_points.is_none()
            && self.font_name.is_none()
            && self.font_color.is_none()
            && self.horizontal_alignment.is_none()
            && self.vertical_alignment.is_none()
            && !self.wrap
            && self
                .fill_pattern
                .is_none_or(|pattern| pattern == Biff8FillPattern::None)
            && self.fill_foreground_color.is_none()
            && self.fill_background_color.is_none()
            && self.number_format.is_none()
    }
}

