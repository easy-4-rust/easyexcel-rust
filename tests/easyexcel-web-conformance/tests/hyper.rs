//! Hyper 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_hyper::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, download_rows, runtime, upload_fixture, verify_download,
    verify_upload,
};
use http::Request;
use http_body_util::{BodyExt, Full};

#[tokio::test]
async fn hyper_conforms_to_shared_excel_contract() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "fixture.csv")
        .header("x-request-id", "hyper-upload")
        .body(Full::new(upload_fixture()))
        .expect("build Hyper request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Hyper upload");
    verify_upload(upload.into_rows())
        .await
        .expect("parse Hyper upload");

    let response = ExcelResponse::prepare(
        download_rows(),
        Format::Xlsx,
        "conformance.xlsx",
        "data",
        runtime.context("hyper-download"),
    )
    .await
    .expect("prepare Hyper download")
    .into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect Hyper body")
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
