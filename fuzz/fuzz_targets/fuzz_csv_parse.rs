//! 模糊测试目标：CSV 工作簿解析。
//!
//! 将任意字节喂入 `easyexcel_csv::read_csv()` 调用，使用默认
//! 选项（自动检测分隔符、开启类型推断）。CSV 解析器在遇到
//! 格式错误的输入时不应 panic，错误应以 `Err(...)` 形式返回。
//!
//! 对应 Java：无直接对应对象；Rust 架构扩展（模糊测试基础设施）

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

use easyexcel_csv::CsvReadOptions;

fuzz_target!(|data: &[u8]| {
    let opts = CsvReadOptions::default();
    let _ = easyexcel_csv::read_csv(Cursor::new(data), &opts);
});
