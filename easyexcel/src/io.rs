//! 与格式无关的电子表格 I/O 契约门面。
//!
//! 这里重导出 [`easyexcel_io`] 的格式识别、流式行协议、资源限制和统一错误类型。

pub use easyexcel_io::{
    Error, Format, ReadMode, ResourceLimits, Result, RowSink, RowSource, StreamCell, StreamInfo,
    WriteMode, looks_like_cfb, looks_like_zip,
};

pub use easyexcel_io::io::{file_utils, gzip_record, io_utils};
