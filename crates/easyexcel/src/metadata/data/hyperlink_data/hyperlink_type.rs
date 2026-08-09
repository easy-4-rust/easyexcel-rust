/// 对应 Java：无直接对应对象；Rust 架构扩展。 Hyperlink type matching Java `HyperlinkData.HyperlinkType`.
///
/// Values mirror Apache POI `HyperlinkType` as used by `EasyExcel` 4.0.3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
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

impl HyperlinkType {
    /// Java `values()` 的声明顺序。
    pub const ALL: [Self; 5] = [Self::None, Self::Url, Self::Document, Self::Email, Self::File];
    /// Java 枚举常量名。
    #[must_use] pub const fn java_name(self) -> &'static str {
        match self {
            Self::None => "NONE", Self::Url => "URL", Self::Document => "DOCUMENT",
            Self::Email => "EMAIL", Self::File => "FILE",
        }
    }
    /// Java `getValue()` 的后端中立值；格式引擎在边界转换为具体超链接类型。
    #[must_use] pub const fn get_value(self) -> Self { self }
}

impl std::str::FromStr for HyperlinkType {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL.into_iter().find(|item| item.java_name() == value)
            .ok_or_else(|| format!("unknown HyperlinkData.HyperlinkType value: {value}"))
    }
}
