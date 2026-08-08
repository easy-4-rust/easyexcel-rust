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

