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

#[cfg(test)]
mod template_hyperlink_type_tests {
    use super::*;

    // --- generation_target 测试 ---

    /// Url 类型直接返回原地址。
    #[test]
    fn generation_target_url_passthrough() {
        assert_eq!(
            TemplateHyperlinkType::Url.generation_target("https://example.com"),
            "https://example.com"
        );
    }

    /// Document 类型在无前缀时加 `internal:`。
    #[test]
    fn generation_target_document_adds_prefix() {
        assert_eq!(
            TemplateHyperlinkType::Document.generation_target("Sheet1!A1"),
            "internal:Sheet1!A1"
        );
    }

    /// Document 类型在已有前缀时保持不变。
    #[test]
    fn generation_target_document_keeps_existing_prefix() {
        assert_eq!(
            TemplateHyperlinkType::Document.generation_target("internal:Sheet1!A1"),
            "internal:Sheet1!A1"
        );
    }

    /// Email 类型在无前缀时加 `mailto:`。
    #[test]
    fn generation_target_email_adds_prefix() {
        assert_eq!(
            TemplateHyperlinkType::Email.generation_target("user@example.com"),
            "mailto:user@example.com"
        );
    }

    /// Email 类型已有 mailto: 前缀时保持不变。
    #[test]
    fn generation_target_email_keeps_existing_prefix() {
        assert_eq!(
            TemplateHyperlinkType::Email.generation_target("mailto:user@example.com"),
            "mailto:user@example.com"
        );
    }

    /// Email 类型大小写不敏感匹配 mailto:。
    #[test]
    fn generation_target_email_case_insensitive_mailto() {
        assert_eq!(
            TemplateHyperlinkType::Email.generation_target("MAILTO:user@example.com"),
            "MAILTO:user@example.com"
        );
    }

    /// File 类型在无前缀时加 `external:`。
    #[test]
    fn generation_target_file_adds_prefix() {
        assert_eq!(
            TemplateHyperlinkType::File.generation_target("/path/to/file"),
            "external:/path/to/file"
        );
    }

    /// File 类型已有前缀时保持不变。
    #[test]
    fn generation_target_file_keeps_existing_prefix() {
        assert_eq!(
            TemplateHyperlinkType::File.generation_target("external:/path/to/file"),
            "external:/path/to/file"
        );
    }

    // --- package_target 测试 ---

    /// Document 类型的 package_target 去掉 `internal:` 前缀。
    #[test]
    fn package_target_document_strips_prefix() {
        assert_eq!(
            TemplateHyperlinkType::Document.package_target("internal:Sheet1!A1"),
            "Sheet1!A1"
        );
    }

    /// Document 类型无前缀时原样返回。
    #[test]
    fn package_target_document_no_prefix_passthrough() {
        assert_eq!(
            TemplateHyperlinkType::Document.package_target("Sheet1!A1"),
            "Sheet1!A1"
        );
    }

    /// Email 类型无 mailto: 前缀时补上。
    #[test]
    fn package_target_email_adds_mailto() {
        assert_eq!(
            TemplateHyperlinkType::Email.package_target("user@example.com"),
            "mailto:user@example.com"
        );
    }

    /// Email 类型已有 mailto: 前缀时保持不变。
    #[test]
    fn package_target_email_keeps_existing_mailto() {
        assert_eq!(
            TemplateHyperlinkType::Email.package_target("mailto:user@example.com"),
            "mailto:user@example.com"
        );
    }

    /// File 类型去掉 `external:` 前缀。
    #[test]
    fn package_target_file_strips_prefix() {
        assert_eq!(
            TemplateHyperlinkType::File.package_target("external:/path/to/file"),
            "/path/to/file"
        );
    }

    /// File 类型无前缀时原样返回。
    #[test]
    fn package_target_file_no_prefix_passthrough() {
        assert_eq!(
            TemplateHyperlinkType::File.package_target("/path/to/file"),
            "/path/to/file"
        );
    }

    /// Url 类型的 package_target 直接返回原地址。
    #[test]
    fn package_target_url_passthrough() {
        assert_eq!(
            TemplateHyperlinkType::Url.package_target("https://example.com"),
            "https://example.com"
        );
    }

    /// 两种方法往返：generation_target → package_target 还原 URL。
    #[test]
    fn roundtrip_url_address() {
        let addr = "https://example.com/path?q=1";
        let generated = TemplateHyperlinkType::Url.generation_target(addr);
        let pkg = TemplateHyperlinkType::Url.package_target(&generated);
        assert_eq!(pkg, addr);
    }

    /// 两种方法往返：Document 内部地址。
    #[test]
    fn roundtrip_document_address() {
        let addr = "Sheet1!A1";
        let generated = TemplateHyperlinkType::Document.generation_target(addr);
        assert_eq!(generated, "internal:Sheet1!A1");
        let pkg = TemplateHyperlinkType::Document.package_target(&generated);
        assert_eq!(pkg, addr);
    }
}
