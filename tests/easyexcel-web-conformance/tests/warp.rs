//! Warp 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_warp::{ExcelRequest, ExcelResponse, excel_request};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, download_rows, runtime, upload_fixture, verify_download,
    verify_upload,
};
use http_body_util::BodyExt;
use warp::Reply;

#[tokio::test]
async fn warp_conforms_to_shared_excel_contract() {
    let runtime = runtime();
    let filter = excel_request::<ConformanceRow>(runtime.clone());
    let upload: ExcelRequest<ConformanceRow> = warp::test::request()
        .method("POST")
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "fixture.csv")
        .header("x-request-id", "warp-upload")
        .body(upload_fixture())
        .filter(&filter)
        .await
        .expect("extract Warp upload");
    verify_upload(upload.into_rows())
        .await
        .expect("parse Warp upload");

    let response = ExcelResponse::prepare(
        download_rows(),
        Format::Xlsx,
        "conformance.xlsx",
        "data",
        runtime.context("warp-download"),
    )
    .await
    .expect("prepare Warp download")
    .into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect Warp body")
        .to_bytes();
    verify_download(&ResponseSnapshot {
        status,
        content_type,
        content_disposition,
        body,
    });
}

fn header(headers: &http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
