/// 对应 Java：无直接对应对象；Rust 架构扩展。 Hyperlink type matching Java `HyperlinkData.HyperlinkType`.
///
/// Values mirror Apache POI `HyperlinkType` as used by `EasyExcel` 4.0.3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HyperlinkType {
    /// Not a hyperlink. (Java `NONE`)
    #[default]
    None,
    /// Link to an existing file or web page. (Java `URL`)
    Url,
    /// Link to a place in this document. (Java `DOCUMENT`)
    Document,
    /// Link to an e-mail address. (Java `EMAIL`)
    Email,
    /// Link to a file. (Java `FILE`)
    File,
}

