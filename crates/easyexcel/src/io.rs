//! 与格式无关的电子表格 I/O 契约门面。
//!
//! 这里重导出 [`easyexcel_io`] 的格式识别、流式行协议、资源限制和统一错误类型。

pub use easyexcel_io::{
    ByteOrderMark, CloseableOutputStream, Error, Format, GzipCellRecordReader,
    GzipCellRecordWriter, GzipCellValue, ReadMode, ResourceLimits, Result, RowSink, RowSource,
    SharedByteBuffer, SheetSelection, StreamCell, StreamInfo, WriteMode, looks_like_cfb,
    looks_like_delimited_text, looks_like_zip, path_has_extension, read_all, select_sheet_names,
    validate_row_range, write_all_and_flush,
};

pub use easyexcel_io::io::{file_utils, gzip_record, http_fetch, io_utils, media_type};
