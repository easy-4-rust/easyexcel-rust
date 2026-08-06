/// 对应 Java：无直接对应对象；Rust 架构扩展。 An opaque part preserved verbatim for round-trip (charts, drawings, pivots,
/// VBA, unknown XML parts or BIFF records).
#[derive(Debug, Clone)]
pub struct OpaquePart {
    /// Logical name/path (zip part name, or OLE stream name).
    pub name: String,
    pub data: Vec<u8>,
}

