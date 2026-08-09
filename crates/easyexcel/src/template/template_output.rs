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
    /// 已由统一 `ExcelWriter` 擦除具体类型的输出流及其关闭动作。
    ///
    /// 这是 Rust 门面对 Java `OutputStream` 生命周期的适配，不复制新的
    /// 流实现；实际写入仍落到调用方原始流，关闭仍由原 builder 回调完成。
    Managed {
        writer: Box<dyn Write + Send + 'a>,
        close: Option<Box<dyn FnOnce() -> std::io::Result<()> + Send + 'a>>,
    },
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
