//! XLS (BIFF8, Excel 97–2003) reader and writer.
//!
//! 公共模型写入入口与 EasyExcel 门面共用 `biff8::Biff8Book` 完整引擎。

mod biff;
mod biff8_sst_string;
mod path_io;
mod reader;
mod sst;
mod writer;

pub use biff8_sst_string::Biff8SstString;
pub use path_io::{
    CFB_MAGIC, looks_like_cfb, read_path, read_path_with_password, write_path,
    write_path_with_password,
};
pub use reader::{
    read, read_decrypted_workbook_stream, read_with_limits, read_with_password,
    read_with_password_and_limits,
};
pub use sst::parse_sst_rich;
pub use writer::{to_biff8_book, write, write_with_password};
