//! CSV 编码写入器。
//!
//! 对应 Java：`com.alibaba.excel.csv.CsvEncodingWriter`。
//! 原文件：easyexcel-core/src/main/java/com/alibaba/excel/csv/CsvEncodingWriter.java

use std::io::Write;

use easyexcel_core::{CsvCharset, Result};
use encoding_rs::{CoderResult, Encoding, UTF_8, UTF_16BE, UTF_16LE};

/// UTF-8 到配置的 CSV 字符集的增量转码器。
///
/// 对应 Java：`com.alibaba.excel.csv.CsvEncodingWriter`。
/// 调用 [`Self::finish`] 以报告不完整的 UTF-8 和编码器终结错误。
pub struct CsvEncodingWriter {
    output: Box<dyn Write + Send>,
    encoder: CsvEncoder,
    pending_utf8: Vec<u8>,
}

/// CSV 编码类型。
#[derive(Clone, Copy)]
pub enum CsvEncoding {
    /// 标准编码。
    Standard(&'static Encoding),
    /// UTF-16 LE 编码。
    Utf16Le,
    /// UTF-16 BE 编码。
    Utf16Be,
}

/// CSV 编码器。
pub enum CsvEncoder {
    /// 使用标准编码（由 `encoding_rs` 驱动）的增量编码器。
    Standard(encoding_rs::Encoder),
    /// UTF-16 LE 编码。
    Utf16Le,
    /// UTF-16 BE 编码。
    Utf16Be,
}

impl CsvEncodingWriter {
    /// 创建转码写入器，使用 Java 风格的字符集名称。
    ///
    /// # Errors
    ///
    /// 当字符集不支持时返回错误。
    pub fn with_charset<W>(output: W, charset: &CsvCharset) -> Result<Self>
    where
        W: Write + Send + 'static,
    {
        Ok(Self::new(Box::new(output), csv_encoding(charset)?))
    }

    pub(crate) fn new(output: Box<dyn Write + Send>, encoding: CsvEncoding) -> Self {
        Self {
            output,
            encoder: match encoding {
                CsvEncoding::Standard(encoding) => CsvEncoder::Standard(encoding.new_encoder()),
                CsvEncoding::Utf16Le => CsvEncoder::Utf16Le,
                CsvEncoding::Utf16Be => CsvEncoder::Utf16Be,
            },
            pending_utf8: Vec::new(),
        }
    }

    fn encode_text(&mut self, text: &str, last: bool) -> std::io::Result<()> {
        match &mut self.encoder {
            CsvEncoder::Standard(encoder) => {
                Self::encode_standard(&mut self.output, encoder, text, last)
            }
            CsvEncoder::Utf16Le => Self::encode_utf16(&mut self.output, text, u16::to_le_bytes),
            CsvEncoder::Utf16Be => Self::encode_utf16(&mut self.output, text, u16::to_be_bytes),
        }
    }

    fn encode_standard(
        output: &mut dyn Write,
        encoder: &mut encoding_rs::Encoder,
        mut text: &str,
        last: bool,
    ) -> std::io::Result<()> {
        loop {
            let mut buffer = [0_u8; 4 * 1_024];
            let (result, read, written, _) = encoder.encode_from_utf8(text, &mut buffer, last);
            output.write_all(&buffer[..written])?;
            text = &text[read..];
            if result == CoderResult::InputEmpty {
                return Ok(());
            }
        }
    }

    /// 将 UTF-16 码元序列按给定的字节序写入输出流。
    ///
    /// # Errors
    ///
    /// 当底层输出写入失败时返回错误。
    pub fn encode_utf16(
        output: &mut dyn Write,
        text: &str,
        to_bytes: fn(u16) -> [u8; 2],
    ) -> std::io::Result<()> {
        let mut encoded = [0_u8; 8 * 1_024];
        let mut length = 0;
        for unit in text.encode_utf16() {
            if length == encoded.len() {
                output.write_all(&encoded)?;
                length = 0;
            }
            let bytes = to_bytes(unit);
            encoded[length] = bytes[0];
            encoded[length + 1] = bytes[1];
            length += 2;
        }
        output.write_all(&encoded[..length])
    }

    /// 终结编码器并刷新底层输出。
    ///
    /// # Errors
    ///
    /// 当 UTF-8 不完整或底层输出失败时返回错误。
    pub fn finish(&mut self) -> std::io::Result<()> {
        if !self.pending_utf8.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "CSV writer ended with incomplete UTF-8",
            ));
        }
        self.encode_text("", true)?;
        self.output.flush()
    }
}

impl Write for CsvEncodingWriter {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        self.pending_utf8.extend_from_slice(buffer);
        let valid_length = match std::str::from_utf8(&self.pending_utf8) {
            Ok(_) => self.pending_utf8.len(),
            Err(error) if error.error_len().is_none() => error.valid_up_to(),
            Err(error) => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, error));
            }
        };
        if valid_length > 0 {
            let valid = self.pending_utf8.drain(..valid_length).collect::<Vec<_>>();
            let text = String::from_utf8_lossy(&valid);
            self.encode_text(text.as_ref(), false)?;
        }
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

/// 将字符集名称解析为对应的 CSV 编码。
///
/// # Errors
///
/// 当字符集不支持时返回错误。
pub fn csv_encoding(charset: &CsvCharset) -> Result<CsvEncoding> {
    let encoding = Encoding::for_label(charset.name().as_bytes()).ok_or_else(|| {
        easyexcel_core::ExcelError::Unsupported(format!(
            "unsupported CSV charset: {}",
            charset.name()
        ))
    })?;
    Ok(if encoding == UTF_16LE {
        CsvEncoding::Utf16Le
    } else if encoding == UTF_16BE {
        CsvEncoding::Utf16Be
    } else {
        CsvEncoding::Standard(encoding)
    })
}

/// 返回给定 CSV 编码对应的 BOM 字节序列。
pub fn csv_bom(encoding: CsvEncoding) -> &'static [u8] {
    match encoding {
        CsvEncoding::Standard(encoding) if encoding == UTF_8 => b"\xEF\xBB\xBF",
        CsvEncoding::Utf16Le => b"\xFF\xFE",
        CsvEncoding::Utf16Be => b"\xFE\xFF",
        CsvEncoding::Standard(_) => b"",
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    use easyexcel_core::CsvCharset;
    use std::io::Cursor;

    #[test]
    fn csv_encoding_writer_with_charset_transcodes() {
        let mut writer =
            CsvEncodingWriter::with_charset(Cursor::new(Vec::<u8>::new()), &CsvCharset::new("GBK"))
                .expect("GBK charset is supported");
        writer
            .write_all("中文,data\n".as_bytes())
            .expect("write must succeed");
        writer.flush().expect("flush must succeed");
    }
}
