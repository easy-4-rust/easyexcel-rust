//! CSV 编码基础引擎的 `EasyExcel` 错误契约适配器。
//!
//! 对应 Java：`com.alibaba.excel.csv.CsvEncodingWriter`。

use std::io::Write;

use crate::core::{CsvCharset, Result};

pub use easyexcel_csv::CsvEncoding;

/// 保持原有 `easyexcel::CsvEncodingWriter` API 的薄适配器。
pub struct CsvEncodingWriter {
    inner: easyexcel_csv::CsvEncodingWriter,
}

impl CsvEncodingWriter {
    /// 使用 Java 风格字符集名称创建转码器。
    ///
    /// # Errors
    ///
    /// 字符集不受支持时返回 `EasyExcel` 公共错误。
    pub fn with_charset<W>(output: W, charset: &CsvCharset) -> Result<Self>
    where
        W: Write + Send + 'static,
    {
        let inner = easyexcel_csv::CsvEncodingWriter::with_charset(output, charset)
            .map_err(crate::core::ExcelError::from)?;
        Ok(Self { inner })
    }

    #[cfg(test)]
    pub(crate) fn new(output: Box<dyn Write + Send>, encoding: CsvEncoding) -> Self {
        Self {
            inner: easyexcel_csv::CsvEncodingWriter::new(output, encoding),
        }
    }

    /// 将 UTF-16 码元按指定字节序写入输出。
    ///
    /// # Errors
    ///
    /// 底层输出无法写入时返回 I/O 错误。
    pub fn encode_utf16(
        output: &mut dyn Write,
        text: &str,
        to_bytes: fn(u16) -> [u8; 2],
    ) -> std::io::Result<()> {
        easyexcel_csv::CsvEncodingWriter::encode_utf16(output, text, to_bytes)
    }

    /// 终结编码器并刷新底层输出。
    ///
    /// # Errors
    ///
    /// 剩余数据不是完整 UTF-8，或底层输出失败时返回 I/O 错误。
    pub fn finish(&mut self) -> std::io::Result<()> {
        self.inner.finish()
    }
}

impl Write for CsvEncodingWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// 将字符集名称解析为具体编码，并保持原有错误类型。
///
/// # Errors
///
/// 字符集名称不受支持时返回 `EasyExcel` 公共错误。
pub fn csv_encoding(charset: &CsvCharset) -> Result<CsvEncoding> {
    easyexcel_csv::csv_encoding(charset).map_err(crate::core::ExcelError::from)
}

/// 返回编码对应的字节顺序标记。
#[must_use]
pub fn csv_bom(encoding: CsvEncoding) -> &'static [u8] {
    easyexcel_csv::csv_bom(encoding)
}
