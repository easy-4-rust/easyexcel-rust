use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue};

/// OOXML 工作簿 MIME 类型。
///
/// 对应 Java：Web 下载响应的 XLSX content type。
pub const XLSX_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// 生成 RFC 5987 XLSX 附件的 `Content-Disposition` 值。
///
/// 对应 Java：`URLEncoder.encode(fileName, UTF_8)` 后设置附件文件名。调用方传入
/// 完整逻辑文件名；空格统一编码为 `%20`。
#[must_use]
pub fn excel_attachment_content_disposition(file_name: &str) -> String {
    let encoded = urlencoding::encode(file_name).replace('+', "%20");
    format!("attachment;filename*=utf-8''{encoded}")
}

/// 生成默认 `.xlsx` 下载响应头。
///
/// `file_name` 不含扩展名；无效 header 值 fail-safe 到 ASCII 下载名。
#[must_use]
pub fn excel_xlsx_attachment_headers(file_name: &str) -> HeaderMap {
    let disposition = excel_attachment_content_disposition(&format!("{file_name}.xlsx"));
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(XLSX_CONTENT_TYPE));
    headers.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment;filename=download.xlsx")),
    );
    headers
}
