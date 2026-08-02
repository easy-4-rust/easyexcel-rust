//! XLSX 附件响应头工具（Tide 0.16 / http-types 2.x 版）。
//!
//! 对应 Java `WebTest.download` 中的：
//! ```java
//! response.setContentType("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
//! response.setHeader("Content-disposition", "attachment;filename*=utf-8''" + fileName + ".xlsx");
//! ```
//!
//! 与 easyexcel-actix 一致，此处返回 `(Content-Type, Content-Disposition)`
//! 头值对而非独立头映射：http-types 2.x 的 `Headers::new()` 为 crate 私有、
//! 且无公开构造器，无法在适配层独立构建 `tide::http::Headers`。

use std::str::FromStr;

use tide::http::headers::{HeaderName, HeaderValue};

/// OOXML 工作簿 MIME 类型。
///
/// 对应 Java `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`。
pub const XLSX_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// `Content-Disposition` 头名（http-types 内置常量中无此项，按 ASCII 头名解析）。
const CONTENT_DISPOSITION: &str = "content-disposition";

/// 生成 XLSX 附件下载所需的响应头值对。
///
/// `file_name` 为不含扩展名的逻辑文件名（Java 侧为 `URLEncoder.encode("测试")` 结果）。
/// `Content-Disposition` 使用 RFC 5987 `filename*` 语法，并将 `+` 替换为 `%20`，
/// 与 Java `WebTest` 保持一致。
///
/// # Panics
///
/// 头值构造失败时 panic（百分号编码后的头值必为 ASCII，正常输入不可达）。
#[must_use]
pub fn excel_xlsx_attachment_headers(file_name: &str) -> (HeaderValue, HeaderValue) {
    let encoded = urlencoding::encode(file_name).replace('+', "%20");
    let disposition = format!("attachment;filename*=utf-8''{encoded}.xlsx");

    // 静态 ASCII 常量与百分号编码结果必为合法 ASCII，parse 恒成功；
    // 回退值仅防御未来编码器变化（与 axum 版 unwrap_or_else 语义一致）。
    let content_type =
        HeaderValue::from_str(XLSX_CONTENT_TYPE).expect("XLSX MIME 常量为合法 ASCII");
    let content_disposition = HeaderValue::from_str(&disposition).unwrap_or_else(|_| {
        HeaderValue::from_str("attachment;filename=download.xlsx").expect("固定回退值为合法 ASCII")
    });
    (content_type, content_disposition)
}

/// 将附件头写入 Tide 响应。
///
/// 对应 Java `WebTest.download` 中的 `setContentType` / `setHeader` 两次调用。
///
/// # Panics
///
/// `Content-Disposition` 头名解析失败时 panic（`"content-disposition"` 为合法 ASCII 头名）。
pub fn apply_excel_xlsx_attachment_headers(response: &mut tide::Response, file_name: &str) {
    let (content_type, content_disposition) = excel_xlsx_attachment_headers(file_name);
    response.insert_header(tide::http::headers::CONTENT_TYPE, content_type);
    response.insert_header(
        HeaderName::from_str(CONTENT_DISPOSITION).expect("content-disposition 为合法 ASCII 头名"),
        content_disposition,
    );
}
