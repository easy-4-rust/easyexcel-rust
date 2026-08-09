/// BIFF8 HLINK 目标类型。
///
/// 对应 Java：Apache POI `HyperlinkType` 的 URL、DOCUMENT、EMAIL 与 FILE。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biff8HyperlinkKind {
    /// Web URL。
    Url,
    /// 当前工作簿内的位置。
    Document,
    /// 电子邮件地址；BIFF8 与 URL 使用相同 URL moniker。
    Email,
    /// 文件系统路径。
    File,
}

impl Biff8HyperlinkKind {
    /// 规范化为 BIFF8 HLINK 编码器接受的目标地址。
    ///
    /// EMAIL 与 URL 共用 URL moniker，因此邮件地址必须包含 `mailto:`；
    /// 其他类型保留调用方原始文本，由对应 moniker/location 编码器处理。
    #[must_use]
    pub fn normalized_target(self, address: &str) -> String {
        if self == Self::Email && !address.to_ascii_lowercase().starts_with("mailto:") {
            format!("mailto:{address}")
        } else {
            address.to_owned()
        }
    }
}
