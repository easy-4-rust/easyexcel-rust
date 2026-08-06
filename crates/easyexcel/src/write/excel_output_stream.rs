//! Java `OutputStream` 语义的 `EasyExcel` 兼容名称。
//!
//! 共享所有权、关闭、刷新和写入实现位于 `easyexcel-io`；门面仅保留既有
//! `ExcelOutputStream<W>` 类型名与构造/方法调用方式。

/// 可克隆、可显式关闭的共享 Excel 输出流。
/// 对应 Java：无直接对应对象；Rust 架构扩展。
pub type ExcelOutputStream<W> = easyexcel_io::CloseableOutputStream<W>;
