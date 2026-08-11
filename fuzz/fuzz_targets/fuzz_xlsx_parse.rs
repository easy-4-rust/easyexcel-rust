//! 模糊测试目标：XLSX（OOXML）工作簿解析。
//!
//! 将任意字节喂入 `easyexcel_xlsx::read()` 调用。解析器在遇到
//! 格式错误的输入时不应 panic，而应返回 `Err(...)`。
//!
//! 对应 Java：无直接对应对象；Rust 架构扩展（模糊测试基础设施）

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // read() 接受任何实现了 Read+Seek 的类型；Cursor<&[u8]> 满足要求。
    let _ = easyexcel_xlsx::read(Cursor::new(data));
});
