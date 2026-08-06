//! CSV、TSV、字符集与流式编解码实现。

mod codec;
mod csv_cell;
mod csv_cell_style;
mod csv_charset;
mod csv_data_format;
mod csv_encoding_writer;
mod csv_rich_text_string;
mod csv_row;
mod csv_sheet;
mod csv_workbook;
mod index;
mod record_io;

pub use codec::{
    CsvReadOptions, CsvWriteOptions, decode_bytes, decode_reader, detect_delimiter, infer_cell,
    read_csv, resolve_encoding, write_csv,
};
pub use csv_cell::{CsvCell, CsvCellValue, CsvNumericCellType};
pub use csv_cell_style::CsvCellStyle;
pub use csv_charset::CsvCharset;
pub use csv_data_format::CsvDataFormat;
pub use csv_encoding_writer::{CsvEncoding, CsvEncodingWriter, csv_bom, csv_encoding};
pub use csv_rich_text_string::CsvRichTextString;
pub use csv_row::CsvRow;
pub use csv_sheet::CsvSheet;
pub use csv_workbook::CsvWorkbook;
pub use index::{checked_column_index, checked_row_index};
pub use record_io::{CsvRecordReader, CsvRecordWriter};
