//! Excel 写入与 Hyper 响应构建。
//!
//! 对应 Java：
//! ```java
//! EasyExcel.write(response.getOutputStream(), DownloadData.class)
//!     .sheet("模板")
//!     .doWrite(data());
//! ```

use easyexcel::core::{ExcelDownloadErrorBody, ExcelError, Result};
use easyexcel::{EasyExcel, ExcelRow};
use http_body_util::Full;
use hyper::{Response, StatusCode};

use crate::headers::excel_xlsx_attachment_headers;

/// Hyper 1.x 适配器的内存响应体类型。
///
/// hyper 1.11 中 `hyper::body::Body` 是 `http_body::Body` trait（非具体类型），
/// 构造响应需用 `http-body-util` 的 [`Full<Bytes>`](http_body_util::Full)
/// 承载内存字节，对应 axum 适配器的 `Body::from(Vec<u8>)`。
pub type ResponseBody = Full<bytes::Bytes>;

/// 将 [`ExcelRow`] 行序列化为 XLSX 字节数组。
///
/// 对应 Java `EasyExcel.write(OutputStream, clazz).sheet(name).doWrite(rows)`，
/// 通过内存 `Vec<u8>` 模拟 `HttpServletResponse.getOutputStream()`。
///
/// # Errors
///
/// 行转换、工作表配置或 OOXML 写入失败时返回 [`easyexcel::core::ExcelError`]。
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

/// 由已生成的 XLSX 字节构建 Hyper 附件响应。
///
/// 对应 Java `WebTest.download` 中 `HttpServletResponse` 的响应头写出，
/// 经 `hyper::Response::builder()` 组装为 `Response<ResponseBody>`。
///
/// # Errors
///
/// 仅在响应状态 / 头构造非法时失败（正常 UTF-8 文件名不会触发）。
pub fn excel_download_response_from_bytes(
    file_name: &str,
    bytes: Vec<u8>,
) -> Result<Response<ResponseBody>> {
    let builder = excel_xlsx_attachment_headers(file_name).into_iter().fold(
        Response::builder().status(StatusCode::OK),
        |builder, (name, value)| match name {
            Some(name) => builder.header(name, value),
            None => builder,
        },
    );
    builder
        .body(Full::from(bytes))
        .map_err(|error| ExcelError::Format(format!("failed to build http response: {error}")))
}

/// 一步完成写入并返回 Hyper XLSX 附件响应。
///
/// 对应 Java `WebTest.download`。
///
/// # Errors
///
/// 写入或响应头构造失败时返回错误。
pub fn excel_download_response<T, I>(
    file_name: &str,
    sheet_name: &str,
    rows: I,
) -> Result<Response<ResponseBody>>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let bytes = write_rows_to_bytes::<T, _>(sheet_name, rows)?;
    excel_download_response_from_bytes(file_name, bytes)
}

/// 下载失败时返回 JSON 体（Hyper 原生响应）。
///
/// 对应 Java `WebTest.downloadFailedUsingJson` 的 `catch` 分支与 Fastjson 输出。
///
/// `ExcelDownloadErrorBody` 按值传入以对齐 Java 的 `errorResponse(body)` 语义，
/// 函数只读该值，按值传递属于 API 契约的一部分，故豁免 `needless_pass_by_value`。
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn excel_download_error_response(body: ExcelDownloadErrorBody) -> Response<ResponseBody> {
    let mut response = Response::new(Full::from(serde_json::to_string(&body).unwrap_or_else(
        |_| r#"{"status":"failure","message":"下载文件失败JSON序列化错误"}"#.to_owned(),
    )));
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    response.headers_mut().insert(
        http::header::CONTENT_TYPE,
        http::HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response
}

/// 尝试生成 XLSX 附件；失败时自动降级为 JSON 错误体。
///
/// 对应 Java `downloadFailedUsingJson` 的整体 try/catch 语义（含
/// `autoCloseStream(false)` 的一次性写入）。
#[must_use]
pub fn excel_download_or_json_response<T, I>(
    file_name: &str,
    sheet_name: &str,
    rows: I,
) -> Response<ResponseBody>
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
