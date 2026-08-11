//! XLSX 附件响应头工具。
//!
//! 对应 Java `WebTest.download` 中的：
//! ```java
//! response.setContentType("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
//! response.setHeader("Content-disposition", "attachment;filename*=utf-8''" + fileName + ".xlsx");
//! ```

use poem::http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use poem::http::{HeaderMap, HeaderValue};

pub use easyexcel_web::XLSX_CONTENT_TYPE;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成 XLSX 附件下载所需的 HTTP 头。
///
/// `file_name` 为不含扩展名的逻辑文件名（Java 侧为 `URLEncoder.encode("测试")` 结果）。
/// 返回的 `Content-Disposition` 使用 RFC 5987 `filename*` 语法，并将 `+` 替换为 `%20`，
/// 与 Java `WebTest` 保持一致。
#[must_use]
pub fn excel_xlsx_attachment_headers(file_name: &str) -> HeaderMap {
    let disposition =
        easyexcel_web::excel_attachment_content_disposition(&format!("{file_name}.xlsx"));

    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static(XLSX_CONTENT_TYPE));
    headers.insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .unwrap_or_else(|_| HeaderValue::from_static("attachment;filename=download.xlsx")),
    );
    headers
}
