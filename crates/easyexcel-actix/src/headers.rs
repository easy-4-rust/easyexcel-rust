//! XLSX 附件响应头工具（Actix-web 版）。

use actix_web::http::header::{self, HeaderValue};

pub use easyexcel_web::XLSX_CONTENT_TYPE;

/// 对应 Java：无直接对应对象；Rust 架构扩展。 生成 Actix-web 可用的 XLSX 附件响应头对。
///
/// 返回 `(Content-Type, Content-Disposition)`，与 Java `WebTest` 的
/// `filename*=utf-8''` 语法一致。
#[must_use]
pub fn excel_xlsx_attachment_headers(file_name: &str) -> (HeaderValue, HeaderValue) {
    let disposition =
        easyexcel_web::excel_attachment_content_disposition(&format!("{file_name}.xlsx"));
    let content_type = HeaderValue::from_static(XLSX_CONTENT_TYPE);
    let content_disposition = HeaderValue::from_str(&disposition)
        .unwrap_or_else(|_| HeaderValue::from_static("attachment;filename=download.xlsx"));
    (content_type, content_disposition)
}

/// 对应 Java：无直接对应对象；Rust 架构扩展。 将附件头写入 Actix [`header::HeaderMap`]。
pub fn apply_excel_xlsx_attachment_headers(headers: &mut header::HeaderMap, file_name: &str) {
    let (content_type, content_disposition) = excel_xlsx_attachment_headers(file_name);
    headers.insert(header::CONTENT_TYPE, content_type);
    headers.insert(header::CONTENT_DISPOSITION, content_disposition);
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn excel_xlsx_attachment_headers_returns_pair() {
        let (ct, cd) = excel_xlsx_attachment_headers("report");
        assert_eq!(ct, XLSX_CONTENT_TYPE);
        let cd_str = cd.to_str().unwrap();
        assert!(cd_str.contains("report.xlsx"), "cd: {cd_str}");
    }

    #[test]
    fn apply_writes_to_header_map() {
        let mut headers = header::HeaderMap::new();
        apply_excel_xlsx_attachment_headers(&mut headers, "test");
        assert!(headers.contains_key(header::CONTENT_TYPE));
        assert!(headers.contains_key(header::CONTENT_DISPOSITION));
    }

    #[test]
    fn xlsx_content_type_constant() {
        assert!(XLSX_CONTENT_TYPE.contains("spreadsheetml"));
    }
}
