//! OOXML 包内单个条目的内存表示。
//!
//! 对应 Java：内部辅助类型（OOXML 包条目封装）。

use zip::CompressionMethod;

#[derive(Debug)]
pub(crate) struct TemplateEntry {
    pub(crate) name: String,
    pub(crate) is_dir: bool,
    pub(crate) compression: CompressionMethod,
    pub(crate) unix_mode: Option<u32>,
    pub(crate) bytes: Vec<u8>,
}
