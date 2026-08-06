/// 对应 Java：无直接对应对象；Rust 架构扩展。 Document metadata (Dublin Core-ish).
#[derive(Debug, Clone, Default)]
pub struct Metadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub company: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub application: Option<String>,
}

