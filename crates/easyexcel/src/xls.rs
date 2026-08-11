//! BIFF8 XLS 基础工作簿引擎门面。

pub use easyexcel_xls::{
    CFB_MAGIC, biff8, looks_like_cfb, read, read_path, read_path_with_password, to_biff8_book,
    write, write_path, write_path_with_password, write_with_password,
};
