//! easyexcel-support axum 适配器（Spring Boot Web 集成层）。
//!
//! 对应 Java `easyexcel-support` 模块（`com.alibaba.excel.support`）
//! 承载的 Spring Boot WebTest 集成模式：
//! `HttpServletResponse` 下载 / 上传模式。

mod error_body;
mod headers;
mod read_upload;
mod write_response;

#[cfg(test)]
mod tests;

pub use error_body::*;
pub use headers::*;
pub use read_upload::*;
pub use write_response::*;
