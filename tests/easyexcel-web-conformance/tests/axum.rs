//! Axum 对共享 Web 契约的实现验证。

use axum::body::Body;
use axum::extract::FromRequest;
use axum::response::IntoResponse;
use easyexcel::io::Format;
use easyexcel_axum::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, download_rows, runtime, upload_fixture, verify_download,
    verify_upload,
};
use http::Request;
use http_body_util::BodyExt;

#[tokio::test]
async fn axum_conforms_to_shared_excel_contract() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "fixture.csv")
        .header("x-request-id", "axum-upload")
        .body(Body::from(upload_fixture()))
        .expect("build Axum request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Axum upload");
    verify_upload(upload.into_rows())
        .await
        .expect("parse Axum upload");

    let response = ExcelResponse::prepare(
        download_rows(),
        Format::Xlsx,
        "conformance.xlsx",
        "data",
        runtime.context("axum-download"),
    )
    .await
    .expect("prepare Axum download")
    .into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect Axum body")
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
