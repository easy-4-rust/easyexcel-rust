/// 对应 Java：无直接对应对象；Rust 架构扩展。 Options controlling CSV writing.
#[derive(Debug, Clone)]
pub struct CsvWriteOptions {
    /// 单字节字段分隔符。
    pub delimiter: u8,
    /// Line terminator (`\n` or `\r\n`).
    pub crlf: bool,
}

impl Default for CsvWriteOptions {
    fn default() -> Self {
        CsvWriteOptions {
            delimiter: b',',
            crlf: false,
        }
    }
}

