//! BIFF8 XLS 工作簿读取、写入和格式识别。

pub mod biff8;
pub mod xls;

pub use xls::{CFB_MAGIC, looks_like_cfb, read, read_path, write, write_path};
