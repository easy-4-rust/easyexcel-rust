//! XLSX 附件响应头工具（Rocket 适配）。
//!
//! 对应 Java `WebTest.download` 中的：
//! ```java
//! response.setContentType("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
//! response.setHeader("Content-disposition", "attachment;filename*=utf-8''" + fileName + ".xlsx");
//! ```

use rocket::http::Header;

/// OOXML 工作簿 MIME 类型。
///
/// 对应 Java `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet`。
pub const XLSX_CONTENT_TYPE: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// 生成 XLSX 附件下载所需的 HTTP 头（Rocket [`Header`] 列表）。
///
/// `file_name` 为不含扩展名的逻辑文件名（Java 侧为 `URLEncoder.encode("测试")` 结果）。
/// 返回的 `Content-Disposition` 使用 RFC 5987 `filename*` 语法，并将 `+` 替换为 `%20`，
/// 与 Java `WebTest` 保持一致。
///
/// Rocket 的 [`Header::new`] 不做值校验（`Cow<str>` 直接承载），
/// 因此无需 Java `setHeader` 之外的兜底逻辑。
#[must_use]
pub fn excel_xlsx_attachment_headers(file_name: &str) -> Vec<Header<'static>> {
    let encoded = urlencoding::encode(file_name).replace('+', "%20");
    let disposition = format!("attachment;filename*=utf-8''{encoded}.xlsx");

    vec![
        Header::new(http::header::CONTENT_TYPE.as_str(), XLSX_CONTENT_TYPE),
        Header::new(http::header::CONTENT_DISPOSITION.as_str(), disposition),
    ]
}
