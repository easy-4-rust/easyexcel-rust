//! easyexcel-support hyper 适配器（Spring Boot Web 集成层）。
//!
//! 对应 Java `easyexcel-support` 模块（`com.alibaba.excel.support`）
//! 承载的 Spring Boot `WebTest` 集成模式：
//! `HttpServletResponse` 下载 / 上传模式。
//!
//! Hyper 是底层 HTTP 库而非 Web 框架，本适配器保持独立薄层：
//! 仅复用 `hyper::Response<hyper::body::Body>` 作为输出类型，
//! 不引入路由 / 中间件（与 thymeleaf-support 的多框架模式一致）。

mod headers;
mod read_upload;
mod write_response;

pub use headers::{XLSX_CONTENT_TYPE, excel_xlsx_attachment_headers};
pub use read_upload::{
    extension_from_path, read_upload_sync, read_upload_with_listener, write_upload_temp,
};
pub use write_response::{
    ResponseBody, excel_download_error_response, excel_download_or_json_response,
    excel_download_response, excel_download_response_from_bytes, write_rows_to_bytes,
};
