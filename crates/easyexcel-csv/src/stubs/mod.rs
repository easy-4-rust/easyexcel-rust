//! STUB 方法集中目录。
//!
//! CSV 格式不支持 Excel 的许多高级特性（样式、合并区域、冻结窗格等），
//! 因此 Rust 实现中保留了大量 no-op 函数以维持与 Java API 的调用兼容性。
//! 本模块将这些 STUB 函数从各源文件集中到此处，便于未来批量处理。

mod cell_stubs;
mod cell_style_stubs;
mod sheet_stubs;
mod workbook_stubs;
