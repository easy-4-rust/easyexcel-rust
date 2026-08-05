//! 电子表格格式识别、流式行协议、资源限制和统一 I/O 错误。

pub mod io;

pub use io::{
    Error, Format, ReadMode, ResourceLimits, Result, RowSink, RowSource, StreamCell, StreamInfo,
    WriteMode, looks_like_cfb, looks_like_zip,
};
