//! 电子表格格式识别、流式行协议、资源限制和统一 I/O 错误。

pub mod io;

pub use io::{
    ByteOrderMark, CloseableOutputStream, Error, Format, ReadMode, ResourceLimits, Result, RowSink,
    RowSource, SharedByteBuffer, SheetSelection, StreamCell, StreamInfo, WriteMode, looks_like_cfb,
    looks_like_delimited_text, looks_like_zip, path_has_extension, row_is_selected,
    select_sheet_names, validate_row_range,
};
pub use io::io_utils::{read_all, write_all_and_flush};
pub use io::gzip_cell_record::{
    GzipCellRecordReader, GzipCellRecordWriter, GzipCellSpillReader, GzipCellSpillSnapshot,
    GzipCellSpillWriter, GzipCellValue,
};
