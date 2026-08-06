use std::io::Write;

use easyexcel_io::{Error, Result};

/// 执行输出字节上限并统计成功写入量。
pub(crate) struct MarkdownOutputGuard<W: Write> {
    writer: W,
    written: u64,
    limit: u64,
}

impl<W: Write> MarkdownOutputGuard<W> {
    pub(crate) const fn new(writer: W, limit: u64) -> Self {
        Self {
            writer,
            written: 0,
            limit,
        }
    }
    pub(crate) const fn written(&self) -> u64 {
        self.written
    }
    pub(crate) fn write_text(&mut self, value: &str) -> Result<()> {
        let bytes = u64::try_from(value.len()).unwrap_or(u64::MAX);
        let actual = self.written.saturating_add(bytes);
        if actual > self.limit {
            return Err(Error::ResourceLimit {
                resource: "output_bytes",
                limit: self.limit,
                actual,
            });
        }
        self.writer.write_all(value.as_bytes())?;
        self.written = actual;
        Ok(())
    }
    pub(crate) fn flush(&mut self) -> Result<()> {
        self.writer.flush().map_err(Error::from)
    }
    pub(crate) fn into_inner(self) -> W {
        self.writer
    }
}
