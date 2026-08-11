//! 模糊测试目标：Markdown GFM 表格解析。
//!
//! 将任意字节喂入 `easyexcel_markdown::read_markdown()` 调用。
//! Markdown 读取器不应 panic —— 非 UTF-8 输入应返回 `Err(...)`，
//! 格式错误的表格应被优雅处理。
//!
//! 对应 Java：无直接对应对象；Rust 架构扩展（模糊测试基础设施）

#![no_main]
use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

use easyexcel_markdown::MarkdownImportOptions;

fuzz_target!(|data: &[u8]| {
    let opts = MarkdownImportOptions::default();
    let _ = easyexcel_markdown::read_markdown(Cursor::new(data), &opts);
});
