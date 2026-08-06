//! 与 `EasyExcel` 门面无关的 CSV 增量字符集转码器。

use std::io::Write;

use easyexcel_io::{Error, Result};
use encoding_rs::{CoderResult, Encoding, UTF_8, UTF_16BE, UTF_16LE};

use super::CsvCharset;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 UTF-8 到目标 CSV 字符集的增量转码器。
pub struct CsvEncodingWriter {
    output: Box<dyn Write + Send>,
    encoder: CsvEncoder,
    pending_utf8: Vec<u8>,
}

include!("csv_encoding_writer/csv_encoding.rs");

enum CsvEncoder {
    Standard(encoding_rs::Encoder),
    Utf16Le,
    Utf16Be,
}

impl CsvEncodingWriter {
    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用 Java 风格字符集名称创建转码器。
    ///
    /// # Errors
    ///
    /// 字符集不受支持时返回 [`Error::Unsupported`]。
    pub fn with_charset<W>(output: W, charset: &CsvCharset) -> Result<Self>
    where
        W: Write + Send + 'static,
    {
        Ok(Self::new(Box::new(output), csv_encoding(charset)?))
    }

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 使用已经解析的编码创建转码器。
    #[must_use]
    pub fn new(output: Box<dyn Write + Send>, encoding: CsvEncoding) -> Self {
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 将 UTF-16 码元按指定字节序写入输出。
    ///
    /// # Errors
    ///
    /// 底层输出无法写入时返回 I/O 错误。
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

    /// 对应 Java：无直接对应对象；Rust 架构扩展。 终结编码器并刷新底层输出。
    ///
    /// # Errors
    ///
    /// 剩余数据不是完整 UTF-8，或底层输出无法写入、刷新时返回 I/O 错误。
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
            Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, error)),
        };
        if valid_length > 0 {
            let valid = self.pending_utf8.drain(..valid_length).collect::<Vec<_>>();
            self.encode_text(String::from_utf8_lossy(&valid).as_ref(), false)?;
        }
        Ok(buffer.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.output.flush()
    }
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将字符集名称解析为具体编码。
///
/// # Errors
///
/// 字符集名称不受支持时返回 [`Error::Unsupported`]。
pub fn csv_encoding(charset: &CsvCharset) -> Result<CsvEncoding> {
    let encoding = Encoding::for_label(charset.name().as_bytes()).ok_or_else(|| {
        Error::Unsupported(format!("unsupported CSV charset: {}", charset.name()))
    })?;
    Ok(if encoding == UTF_16LE {
        CsvEncoding::Utf16Le
    } else if encoding == UTF_16BE {
        CsvEncoding::Utf16Be
    } else {
        CsvEncoding::Standard(encoding)
    })
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 返回编码对应的字节顺序标记。
#[must_use]
pub fn csv_bom(encoding: CsvEncoding) -> &'static [u8] {
    match encoding {
        CsvEncoding::Standard(encoding) if encoding == UTF_8 => b"\xEF\xBB\xBF",
        CsvEncoding::Utf16Le => b"\xFF\xFE",
        CsvEncoding::Utf16Be => b"\xFE\xFF",
        CsvEncoding::Standard(_) => b"",
    }
}
