// 模板超链接类型枚举。
// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。
// 从 `template_hyperlink.rs` 拆分而来，遵循"一个 .rs 文件只对应一个 Java 对象"规范。

/// 对应 Java：无直接对应对象；Rust XLSX 引擎扩展。模板超链接类型。
///
/// 区分 URL、工作簿内部位置、邮件地址和外部文件四种超链接类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateHyperlinkType {
    /// 普通 URL。
    Url,
    /// 工作簿内部位置。
    Document,
    /// 邮件地址。
    Email,
    /// 外部文件。
    File,
}

impl TemplateHyperlinkType {
    /// 将调用方地址规范化为生成式 XLSX 后端接受的目标。
    ///
    /// 工作簿内部、邮件与外部文件链接分别使用 `internal:`、`mailto:` 与
    /// `external:` 前缀；已经带有前缀的地址保持不变。
    ///
    /// # 参数
    /// - `address`: 原始地址字符串。
    ///
    /// # 返回
    /// 规范化后的目标地址。
    #[must_use]
    pub fn generation_target(self, address: &str) -> String {
        match self {
            Self::Url => address.to_owned(),
            Self::Document if address.starts_with("internal:") => address.to_owned(),
            Self::Document => format!("internal:{address}"),
            Self::Email if address.to_ascii_lowercase().starts_with("mailto:") => {
                address.to_owned()
            }
            Self::Email => format!("mailto:{address}"),
            Self::File if address.starts_with("external:") => address.to_owned(),
            Self::File => format!("external:{address}"),
        }
    }

    /// 将地址规范化为 OOXML relationship/location 中保存的目标。
    ///
    /// # 参数
    /// - `address`: 原始地址字符串。
    ///
    /// # 返回
    /// 规范化后的包目标地址。
    #[must_use]
    pub fn package_target(self, address: &str) -> String {
        match self {
            Self::Document => address
                .strip_prefix("internal:")
                .unwrap_or(address)
                .to_owned(),
            Self::Email if !address.to_ascii_lowercase().starts_with("mailto:") => {
                format!("mailto:{address}")
            }
            Self::File => address
                .strip_prefix("external:")
                .unwrap_or(address)
                .to_owned(),
            Self::Url | Self::Email => address.to_owned(),
        }
    }
}
