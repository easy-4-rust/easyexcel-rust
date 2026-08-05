//! CSV、TSV、字符集检测、类型推断和流式编解码。

pub mod csv;

pub use csv::{
    CsvCharset, CsvEncoding, CsvEncodingWriter, CsvReadOptions, CsvWriteOptions, csv_bom,
    csv_encoding, decode_bytes, decode_reader, detect_delimiter, infer_cell, read_csv,
    resolve_encoding, write_csv,
};
