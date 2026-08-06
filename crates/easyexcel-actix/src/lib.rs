//! easyexcel-support actix 适配器（Quarkus Web 集成层）。
//!
//! API 与 `easyexcel-axum` 对称，对应同一套 Java `WebTest` 示例。

mod error_body;
mod headers;
mod read_upload;
mod write_response;

#[cfg(test)]
mod tests;

pub use error_body::ExcelDownloadErrorBody;
pub use headers::{
    XLSX_CONTENT_TYPE, apply_excel_xlsx_attachment_headers, excel_xlsx_attachment_headers,
};
pub use read_upload::{
    extension_from_path, read_upload_sync, read_upload_with_listener, write_upload_temp,
};
pub use write_response::{
    excel_download_error_response, excel_download_or_json_response, excel_download_response,
    excel_download_response_from_bytes, write_rows_to_bytes,
};
