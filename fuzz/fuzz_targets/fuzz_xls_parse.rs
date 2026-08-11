//! 模糊测试目标：XLS（BIFF8）工作簿解析。
//!
//! 将任意字节喂入 `easyexcel_xls::read()` 调用。CFB + BIFF8
//! 解析器在遇到格式错误的输入时不应 panic。
//!
//! 对应 Java：无直接对应对象；Rust 架构扩展（模糊测试基础设施）

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let _ = easyexcel_xls::read(Cursor::new(data));
});
