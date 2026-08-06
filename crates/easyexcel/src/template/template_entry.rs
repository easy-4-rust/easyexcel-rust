//! OOXML 模板条目兼容别名。
//!
//! ZIP 条目模型及重新打包能力由 `easyexcel-xlsx` 维护。

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) type TemplateEntry = easyexcel_xlsx::OoxmlZipEntry;
