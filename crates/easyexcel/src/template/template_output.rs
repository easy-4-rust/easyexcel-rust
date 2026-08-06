//! 模板写入的输出目标抽象与流 trait。
//!
//! 对应 Java：内部辅助类型（输出目标抽象）。

#[cfg(test)]
use std::any::Any;
#[cfg(test)]
use std::io::Seek;
use std::io::Write;
use std::path::PathBuf;

use crate::write::ExcelOutputStream;

/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub(crate) enum TemplateOutput<'a> {
    Path(PathBuf),
    Borrowed(&'a mut dyn Write),
    Owned(Box<dyn CloseableWrite + 'a>),
}

include!("template_output/closeable_write.rs");

#[cfg(test)]
pub(crate) trait WriteSeek: Write + Seek + Any {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

#[cfg(test)]
impl<T: Write + Seek + Any> WriteSeek for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
