//! CSV 基础读写能力门面。
//!
//! 这里重导出 [`easyexcel_csv`] 的公共 API，使外部用户只需依赖
//! `easyexcel` 即可使用 CSV 编解码、字符集处理和类型推断能力。

pub use easyexcel_csv::{
    CsvCellStyle, CsvCellValue, CsvCharset, CsvDataFormat, CsvEncoding, CsvEncodingWriter,
    CsvNumericCellType, CsvReadOptions, CsvRecordReader, CsvRecordWriter, CsvRichTextString,
    CsvWriteOptions, checked_column_index, checked_row_index, csv_bom, csv_encoding, decode_bytes,
    decode_reader, detect_delimiter, infer_cell, read_csv, resolve_encoding, write_csv,
};
pub use crate::metadata::csv::{CsvCell, CsvRow, CsvSheet, CsvWorkbook};
