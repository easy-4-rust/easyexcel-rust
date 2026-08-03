//! Excel 写入与 Poem 响应构建。
//!
//! 对应 Java：
//! ```java
//! EasyExcel.write(response.getOutputStream(), DownloadData.class)
//!     .sheet("模板")
//!     .doWrite(data());
//! ```

use easyexcel::{EasyExcel, ExcelRow};
use easyexcel_core::{ExcelDownloadErrorBody, Result};
use poem::http::StatusCode;
use poem::{Body, Response};

use crate::headers::excel_xlsx_attachment_headers;

/// 将 [`ExcelRow`] 行序列化为 XLSX 字节数组。
///
/// 对应 Java `EasyExcel.write(OutputStream, clazz).sheet(name).doWrite(rows)`，
/// 通过内存 `Vec<u8>` 模拟 `HttpServletResponse.getOutputStream()`。
///
/// # Errors
///
/// 行转换、工作表配置或 OOXML 写入失败时返回 [`easyexcel_core::ExcelError`]。
pub fn write_rows_to_bytes<T, I>(sheet_name: &str, rows: I) -> Result<Vec<u8>>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let mut buffer = Vec::new();
    EasyExcel::write::<T>("download.xlsx")
        .sheet(sheet_name)
        .to_writer(&mut buffer)
        .do_write(rows)?;
    Ok(buffer)
}

/// 由已生成的 XLSX 字节构建 Poem 附件响应。
///
/// # Errors
///
/// 仅在响应头构造非法时失败（正常 UTF-8 文件名不会触发）。
pub fn excel_download_response_from_bytes(file_name: &str, bytes: Vec<u8>) -> Result<Response> {
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(bytes));
    *response.headers_mut() = excel_xlsx_attachment_headers(file_name);
    Ok(response)
}

/// 一步完成写入并返回 Poem XLSX 附件响应。
///
/// 对应 Java `WebTest.download`。
///
/// # Errors
///
/// 写入或响应头构造失败时返回错误。
pub fn excel_download_response<T, I>(file_name: &str, sheet_name: &str, rows: I) -> Result<Response>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let bytes = write_rows_to_bytes::<T, _>(sheet_name, rows)?;
    excel_download_response_from_bytes(file_name, bytes)
}

/// 下载失败时返回 JSON 体（Poem [`Response`]）。
///
/// 对应 Java `WebTest.downloadFailedUsingJson` 的 `catch` 分支与 Fastjson 输出。
///
/// `ExcelDownloadErrorBody` 按值传入以对齐 Java 的 `errorResponse(body)` 语义，
/// 函数只读该值，按值传递属于 API 契约的一部分，故豁免 `needless_pass_by_value`。
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn excel_download_error_response(body: ExcelDownloadErrorBody) -> Response {
    let json = serde_json::to_string(&body).unwrap_or_else(|_| {
        r#"{"status":"failure","message":"下载文件失败JSON序列化错误"}"#.to_owned()
    });
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("application/json; charset=utf-8")
        .body(Body::from_string(json))
}

/// 尝试生成 XLSX 附件；失败时自动降级为 JSON 错误体。
///
/// 对应 Java `downloadFailedUsingJson` 的整体 try/catch 语义（含
/// `autoCloseStream(false)` 的一次性写入）。
#[must_use]
pub fn excel_download_or_json_response<T, I>(file_name: &str, sheet_name: &str, rows: I) -> Response
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    match write_rows_to_bytes::<T, _>(sheet_name, rows) {
        Ok(bytes) => excel_download_response_from_bytes(file_name, bytes).unwrap_or_else(|error| {
            excel_download_error_response(ExcelDownloadErrorBody::download_failed(&error))
        }),
        Err(error) => {
            excel_download_error_response(ExcelDownloadErrorBody::download_failed(&error))
        }
    }
}
