/// 对应 Java：无直接对应对象；Rust 架构扩展。 How the referenced sheet(s) are specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SheetSpec {
    /// No sheet qualifier — resolves to the formula's own sheet.
    Current,
    /// `SheetName!` qualifier.
    Name(String),
    /// 3D span `First:Last!` qualifier.
    Span(String, String),
}

