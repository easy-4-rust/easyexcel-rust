//! CSV、TSV、字符集检测、类型推断和流式编解码。

#![allow(missing_docs)]

pub mod csv;
mod stubs;

pub use csv::{
    CsvCell, CsvCellStyle, CsvCellType, CsvCellValue, CsvCharset, CsvDataFormat, CsvEncoding,
    CsvEncodingWriter, CsvNumericCellType, CsvReadOptions, CsvRecordReader, CsvRecordWriter,
    CsvRichTextString, CsvRow, CsvRowSource, CsvSheet, CsvWorkbook, CsvWriteOptions,
    checked_column_index, checked_row_index, csv_bom, csv_encoding, decode_bytes, decode_reader,
    detect_delimiter, infer_cell, read_csv, resolve_encoding, write_csv,
};
