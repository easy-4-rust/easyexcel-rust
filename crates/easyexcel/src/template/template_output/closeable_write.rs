/// 对应 Java：无直接对应对象；Rust 架构扩展。
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

