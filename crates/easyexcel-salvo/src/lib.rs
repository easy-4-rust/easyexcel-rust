//! `easyexcel-web` 的 Salvo 原生 extractor 与 writer 适配层。

mod excel_error;
mod excel_request;
mod excel_response;
mod headers;

pub use easyexcel_web::{ExcelProblemDetails, ExcelWebPolicy, ExcelWebRuntime};
pub use excel_error::ExcelSalvoError;
pub use excel_request::ExcelRequest;
pub use excel_response::ExcelResponse;
pub use headers::{XLSX_CONTENT_TYPE, excel_xlsx_attachment_headers};
