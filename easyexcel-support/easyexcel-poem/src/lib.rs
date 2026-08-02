//! easyexcel-support poem 适配器（Spring Boot Web 集成层）。
//!
//! 对应 Java `easyexcel-support` 模块（`com.alibaba.excel.support`）
//! 承载的 Spring Boot WebTest 集成模式：
//! `HttpServletResponse` 下载 / 上传模式在 Poem 框架下的适配。

mod headers;
mod read_upload;
mod write_response;

pub use headers::*;
pub use read_upload::*;
pub use write_response::*;
