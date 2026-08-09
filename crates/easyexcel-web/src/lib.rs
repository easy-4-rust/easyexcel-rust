//! `EasyExcel` 的框架中立 Web 集成内核。
//!
//! 本 crate 负责统一上传、下载、资源限制、背压、取消、超时、临时文件生命周期
//! 和稳定错误协议。Axum、Actix Web 等框架适配层只需要完成传输类型转换。

#![forbid(unsafe_code)]

pub mod web;

pub use web::{
    ExcelExport, ExcelImport, ExcelProblemDetails, ExcelRequestMetadata, ExcelRows, ExcelWebError,
    ExcelWebErrorCode, ExcelWebPolicy, ExcelWebRuntime, WebExecutionContext, XLSX_CONTENT_TYPE,
    excel_attachment_content_disposition, excel_xlsx_attachment_headers,
};
