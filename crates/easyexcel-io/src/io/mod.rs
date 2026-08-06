//! 格式识别、流式协议、执行模式、资源限制与统一错误。

mod byte_order_mark;
mod closeable_output_stream;
mod error;
pub mod file_utils;
mod format;
pub mod gzip_cell_record;
pub mod gzip_record;
pub mod http_fetch;
pub mod io_utils;
pub mod media_type;
mod read_mode;
mod resource_limits;
mod row_range;
mod row_sink;
mod row_source;
mod shared_byte_buffer;
mod sheet_selection;
mod stream_cell;
mod stream_info;
mod write_mode;

pub use byte_order_mark::ByteOrderMark;
pub use closeable_output_stream::CloseableOutputStream;
pub use error::{Error, Result};
pub use format::{
    Format, looks_like_cfb, looks_like_delimited_text, looks_like_zip, path_has_extension,
};
pub use read_mode::ReadMode;
pub use resource_limits::ResourceLimits;
pub use row_range::{row_is_selected, validate_row_range};
pub use row_sink::RowSink;
pub use row_source::RowSource;
pub use shared_byte_buffer::SharedByteBuffer;
pub use sheet_selection::{SheetSelection, select_sheet_names};
pub use stream_cell::StreamCell;
pub use stream_info::StreamInfo;
pub use write_mode::WriteMode;
