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

pub(crate) enum TemplateOutput<'a> {
    Path(PathBuf),
    Borrowed(&'a mut dyn Write),
    Owned(Box<dyn CloseableWrite + 'a>),
}

pub(crate) trait CloseableWrite: Write {
    fn close(&self) -> std::io::Result<()>;
}

impl<W> CloseableWrite for ExcelOutputStream<W>
where
    W: Write,
{
    fn close(&self) -> std::io::Result<()> {
        ExcelOutputStream::close(self)
    }
}

pub(crate) use easyexcel_xlsx::xlsx::ReadSeek;

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
