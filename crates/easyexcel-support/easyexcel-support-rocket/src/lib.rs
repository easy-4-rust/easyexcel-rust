//! easyexcel-support rocket 适配器（Spring Boot Web 集成层）。
//!
//! 对应 Java `easyexcel-support` 模块（`com.alibaba.excel.support`）
//! 承载的 Spring Boot `WebTest` 集成模式：
//! `HttpServletResponse` 下载 / 上传模式。
//!
//! 本 crate 以 Rocket（0.5.1）为 Web 框架实现同一套 API 形状，
//! 与 `easyexcel-support-axum` 适配器保持 `1:1` 对齐：
//! `excel_xlsx_attachment_headers` / `excel_download_response*` /
//! `read_upload_*` 分别对应 Java `WebTest` 的 `download` /
//! `downloadFailedUsingJson` / `upload` 语义。

mod headers;
mod read_upload;
mod write_response;

#[cfg(test)]
mod tests;

pub use headers::*;
pub use read_upload::*;
pub use write_response::*;
