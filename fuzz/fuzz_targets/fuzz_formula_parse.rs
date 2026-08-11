//! 模糊测试目标：Excel 公式解析。
//!
//! 将任意字符串喂入 `easyexcel_formula::parse_detailed()` 调用。
//! 公式词法/语法解析器在遇到格式错误的输入时不应 panic，
//! 错误应以 `Err(...)` 形式返回。
//!
//! 对应 Java：无直接对应对象；Rust 架构扩展（模糊测试基础设施）

#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // 将字节转换为字符串（有损）；解析器必须处理任意 UTF-8 文本。
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = easyexcel_formula::parse_detailed(s);
    }
});
