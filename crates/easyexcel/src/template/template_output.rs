//! 模板写入的输出目标抽象与流 trait。
//!
//! 对应 Java：内部辅助类型（输出目标抽象）。

use std::any::Any;
use std::io::{Read, Seek, Write};
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

pub(crate) trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

pub(crate) trait WriteSeek: Write + Seek + Any {
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Write + Seek + Any> WriteSeek for T {
    fn into_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}
