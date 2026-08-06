//! `easyexcel-web` 的 Warp 原生 filter 与 reply 适配层。

mod excel_error;
mod excel_request;
mod excel_response;
mod headers;

pub use easyexcel_web::{ExcelProblemDetails, ExcelWebPolicy, ExcelWebRuntime};
pub use excel_error::{ExcelWarpRejection, recover_excel_rejection};
pub use excel_request::{ExcelRequest, excel_request};
pub use excel_response::ExcelResponse;
pub use headers::{XLSX_CONTENT_TYPE, excel_xlsx_attachment_headers};
