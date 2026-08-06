/// 对应 Java：无直接对应对象；Rust 架构扩展。 A complete cell style. Equality/hash are used to deduplicate styles when
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
    #[must_use]
    /// 对应 Java：无直接对应对象；Rust 架构扩展。
    pub fn is_date(&self) -> bool {
        if let Some(id) = self.number_format_id
            && super::numfmt::is_date_format_id(id, Some(&self.number_format))
        {
            return true;
        }
        super::numfmt::is_date_format(&self.number_format)
    }
}

