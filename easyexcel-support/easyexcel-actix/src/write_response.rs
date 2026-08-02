//! Excel 写入与 Actix-web 响应构建。

use actix_web::HttpResponse;
use easyexcel::{EasyExcel, ExcelRow};
use easyexcel_core::{ExcelDownloadErrorBody, Result};
use serde_json;

/// 将 [`ExcelRow`] 行序列化为 XLSX 字节数组。
///
/// # Errors
///
/// 行转换或 OOXML 写入失败时返回 [`easyexcel_core::ExcelError`]。
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

/// 由 XLSX 字节构建 Actix 附件响应。
#[must_use]
pub fn excel_download_response_from_bytes(file_name: &str, bytes: Vec<u8>) -> HttpResponse {
    let (content_type, content_disposition) =
        crate::headers::excel_xlsx_attachment_headers(file_name);
    HttpResponse::Ok()
        .insert_header((actix_web::http::header::CONTENT_TYPE, content_type))
        .insert_header((
            actix_web::http::header::CONTENT_DISPOSITION,
            content_disposition,
        ))
        .body(bytes)
}

/// 一步完成写入并返回 Actix XLSX 附件响应。
///
/// # Errors
///
/// 写入失败时返回错误（由调用方决定降级策略）。
pub fn excel_download_response<T, I>(
    file_name: &str,
    sheet_name: &str,
    rows: I,
) -> Result<HttpResponse>
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    let bytes = write_rows_to_bytes::<T, _>(sheet_name, rows)?;
    Ok(excel_download_response_from_bytes(file_name, bytes))
}

/// 下载失败时返回 JSON 体。
#[must_use]
pub fn excel_download_error_response(body: ExcelDownloadErrorBody) -> HttpResponse {
    HttpResponse::InternalServerError()
        .content_type("application/json; charset=utf-8")
        .body(serde_json::to_string(&body).unwrap_or_else(|_| {
            r#"{"status":"failure","message":"下载文件失败JSON序列化错误"}"#.to_owned()
        }))
}

/// 尝试生成 XLSX 附件；失败时自动降级为 JSON 错误体。
#[must_use]
pub fn excel_download_or_json_response<T, I>(
    file_name: &str,
    sheet_name: &str,
    rows: I,
) -> HttpResponse
where
    T: ExcelRow,
    I: IntoIterator<Item = T>,
{
    match write_rows_to_bytes::<T, _>(sheet_name, rows) {
        Ok(bytes) => excel_download_response_from_bytes(file_name, bytes),
        Err(error) => {
            excel_download_error_response(ExcelDownloadErrorBody::download_failed(&error))
        }
    }
}

#[cfg(test)]
mod tests_extra2 {
    use super::*;
    use actix_web::http::StatusCode;
    use serde_json::{Value, json};

    /// 读取 Actix 响应体字节（测试辅助）。
    fn response_bytes(response: HttpResponse) -> Vec<u8> {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(async move {
                let body: actix_web::body::BoxBody = response.into_body();
                let bytes = actix_web::body::to_bytes(body).await.expect("body");
                bytes.to_vec()
            })
    }

    /// 对应 Java：尝试触发 `excel_download_error_response` 的 JSON 序列化失败回退分支
    /// （write_response.rs 64-65 行）。
    ///
    /// `ExcelDownloadErrorBody` 仅由两个 `String` 字段派生 `Serialize`，序列化在数学上
    /// 不可能失败，因此 `unwrap_or_else` 的回退文案分支不可达。此处用边界字符串
    /// （换行 / 制表符 / emoji）再次确认回退文案「JSON序列化错误」不会被输出。
    #[test]
    fn error_response_serialization_fallback_is_unreachable() {
        let body = ExcelDownloadErrorBody::download_failed("尝试触发回退分支\n\t🎉");
        let response = excel_download_error_response(body);
        assert_eq!(
            response.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected status"
        );
        assert_eq!(
            response
                .headers()
                .get("content-type")
                .map(|value| value.to_str().unwrap()),
            Some("application/json; charset=utf-8")
        );
        let value: Value = serde_json::from_slice(&response_bytes(response)).expect("json");
        assert_eq!(value["status"], json!("failure"));
        assert_eq!(
            value["message"],
            json!("下载文件失败尝试触发回退分支\n\t🎉")
        );
    }
}
