//! Excel 写入与 Tide 响应构建。
//!
//! 对应 Java：
//! ```java
//! EasyExcel.write(response.getOutputStream(), DownloadData.class)
//!     .sheet("模板")
//!     .doWrite(data());
//! ```

use easyexcel::core::{ExcelDownloadErrorBody, Result};
use easyexcel::{EasyExcel, ExcelRow};

use crate::headers::apply_excel_xlsx_attachment_headers;

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

/// 由已生成的 XLSX 字节构建 Tide 附件响应。
///
/// # Errors
///
/// 仅在响应头构造非法时失败（正常 UTF-8 文件名不会触发）。
pub fn excel_download_response_from_bytes(
    file_name: &str,
    bytes: Vec<u8>,
) -> Result<tide::Response> {
    let mut response = tide::Response::new(tide::StatusCode::Ok);
    apply_excel_xlsx_attachment_headers(&mut response, file_name);
    response.set_body(bytes);
    Ok(response)
}

/// 一步完成写入并返回 Tide XLSX 附件响应。
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
) -> Result<tide::Response>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let bytes = write_rows_to_bytes::<T, _>(sheet_name, rows)?;
    excel_download_response_from_bytes(file_name, bytes)
}

/// 下载失败时返回 JSON 体（Tide [`tide::Response`]）。
///
/// 对应 Java `WebTest.downloadFailedUsingJson` 的 `catch` 分支与 Fastjson 输出。
///
/// `ExcelDownloadErrorBody` 按值传入以对齐 Java 的 `errorResponse(body)` 语义，
/// 函数只读该值，按值传递属于 API 契约的一部分，故豁免 `needless_pass_by_value`。
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn excel_download_error_response(body: ExcelDownloadErrorBody) -> tide::Response {
    let mut response = tide::Response::new(tide::StatusCode::InternalServerError);
    response.insert_header(
        tide::http::headers::CONTENT_TYPE,
        "application/json; charset=utf-8",
    );
    response.set_body(serde_json::to_string(&body).unwrap_or_else(|_| {
        r#"{"status":"failure","message":"下载文件失败JSON序列化错误"}"#.to_owned()
    }));
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
) -> tide::Response
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

#[cfg(test)]
mod tests_extra2 {
    use super::*;
    use serde_json::{Value, json};

    /// 读取 Tide 响应体为字节（测试辅助，`block_on` `tide::Body::into_bytes`）。
    fn body_bytes(body: tide::Body) -> Vec<u8> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(async move { body.into_bytes().await.expect("read body") })
    }

    /// 对应 Java：尝试触发 `excel_download_error_response` 的 JSON 序列化失败回退分支
    /// （`write_response.rs` 92-94 行）。
    ///
    /// `ExcelDownloadErrorBody` 仅由两个 `String` 字段派生 `Serialize`，序列化在数学上
    /// 不可能失败，因此 `unwrap_or_else` 的回退文案分支不可达。此处用边界字符串
    /// （换行 / 制表符 / emoji）再次确认回退文案「JSON序列化错误」不会被输出。
    #[test]
    fn error_response_serialization_fallback_is_unreachable() {
        let body = ExcelDownloadErrorBody::download_failed("尝试触发回退分支\n\t🎉");
        let mut response = excel_download_error_response(body);
        assert_eq!(response.status(), tide::StatusCode::InternalServerError);
        assert_eq!(response["content-type"], "application/json; charset=utf-8");
        let value: Value = serde_json::from_slice(&body_bytes(response.take_body())).expect("json");
        assert_eq!(value["status"], json!("failure"));
        assert_eq!(
            value["message"],
            json!("下载文件失败尝试触发回退分支\n\t🎉")
        );
    }

    /// 对应 Java：尝试让 `excel_download_response_from_bytes` 失败以触发
    /// `excel_download_or_json_response` 成功分支的降级闭包（`write_response.rs` 118-119 行）。
    ///
    /// 文件名经 `urlencoding` 百分号编码后必为合法 ASCII `HeaderValue`，函数恒返回 Ok，
    /// 降级闭包在数学上不可达。此处以边界文件名（中文 / 空格 / emoji / 制表符 / 百分号）
    /// 逐一确认恒成功，且 Content-Disposition 始终为 RFC 5987 `filename*` 形态。
    #[test]
    fn bytes_response_never_fails_for_edge_case_file_names() {
        for name in ["edge case 文件", "emoji🎉", "a\tb", "100%"] {
            let response =
                excel_download_response_from_bytes(name, vec![1, 2, 3]).expect("must succeed");
            assert_eq!(response.status(), tide::StatusCode::Ok);
            let expected = format!(
                "attachment;filename*=utf-8''{}.xlsx",
                urlencoding::encode(name).replace('+', "%20")
            );
            assert_eq!(response["content-disposition"], expected);
        }
    }

    /// 对应 Java：端到端尝试触发 `excel_download_or_json_response` 的字节响应降级闭包。
    ///
    /// 写入成功路径恒走到 `excel_download_response_from_bytes`（恒 Ok），
    /// 与 `adapter_contract.rs` 的 `excel_download_or_json_response_success_path` 互补：
    /// 此处特意使用边界文件名，确认即便文件名含空格也走附件响应而非 JSON 错误体。
    #[derive(Debug, Clone, ExcelRow)]
    struct AttemptRow {
        #[excel(name = "值", order = 1)]
        value: String,
    }

    #[test]
    fn or_json_success_path_stays_attachment_for_edge_case_names() {
        let response = excel_download_or_json_response(
            "edge case 文件",
            "模板",
            [AttemptRow {
                value: "attempt".to_owned(),
            }],
        );
        assert_eq!(response.status(), tide::StatusCode::Ok);
        assert!(response["content-type"].as_str().contains("spreadsheetml"));
        assert!(response["content-disposition"].as_str().contains("utf-8''"));
    }
}
