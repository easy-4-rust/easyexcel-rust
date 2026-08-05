//! CSV、TSV、字符集与流式编解码实现。

mod codec;
mod csv_charset;
mod csv_encoding_writer;
mod record_io;

pub use codec::{
    CsvReadOptions, CsvWriteOptions, decode_bytes, decode_reader, detect_delimiter, infer_cell,
    read_csv, resolve_encoding, write_csv,
};
pub use csv_charset::CsvCharset;
pub use csv_encoding_writer::{CsvEncoding, CsvEncodingWriter, csv_bom, csv_encoding};
pub use record_io::{CsvRecordReader, CsvRecordWriter};
