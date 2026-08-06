/// 对应 Java：无直接对应对象；Rust 架构扩展。 已解析的 CSV 编码类型。
#[derive(Clone, Copy)]
pub enum CsvEncoding {
    /// `encoding_rs` 支持的标准编码。
    Standard(&'static Encoding),
    /// UTF-16 小端编码。
    Utf16Le,
    /// UTF-16 大端编码。
    Utf16Be,
}

