//! easyexcel-support tide 适配器（Tide 0.16 / http-types 2.x Web 集成层）。
//!
//! 对应 Java `easyexcel-support` 模块（`com.alibaba.excel.support`）
//! 承载的 Web 集成模式：`HttpServletResponse` 下载 / 上传模式，
//! 此处以 Tide 的 `tide::Response` / `tide::Body` 呈现。

mod error_body;
mod headers;
mod read_upload;
mod write_response;

pub use error_body::*;
pub use headers::*;
pub use read_upload::*;
pub use write_response::*;
