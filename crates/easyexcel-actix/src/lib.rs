//! `easyexcel-web` 的 Actix Web 原生 extractor 与 responder 适配层。

mod excel_error;
mod excel_request;
mod excel_response;
mod headers;

pub use easyexcel_web::{ExcelProblemDetails, ExcelWebPolicy, ExcelWebRuntime};
pub use excel_error::ExcelActixError;
pub use excel_request::ExcelRequest;
pub use excel_response::ExcelResponse;
pub use headers::{
    XLSX_CONTENT_TYPE, apply_excel_xlsx_attachment_headers, excel_xlsx_attachment_headers,
};
