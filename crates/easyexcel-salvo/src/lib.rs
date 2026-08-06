//! easyexcel-support salvo 适配器（Salvo Web 集成层）。
//!
//! 对应 Java `easyexcel-support` 模块（`com.alibaba.excel.support`）
//! 承载的 Spring Boot `WebTest` 集成模式：
//! `HttpServletResponse` 下载 / 上传模式。

mod headers;
mod read_upload;
mod write_response;

pub use headers::{XLSX_CONTENT_TYPE, excel_xlsx_attachment_headers};
pub use read_upload::{
    extension_from_path, read_upload_sync, read_upload_with_listener, write_upload_temp,
};
pub use write_response::{
    excel_download_error_response, excel_download_or_json_response, excel_download_response,
    excel_download_response_from_bytes, write_rows_to_bytes,
};
