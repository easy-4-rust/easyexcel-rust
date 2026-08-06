//! Poem 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_poem::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, download_rows, runtime, upload_fixture, verify_download,
    verify_upload,
};
use poem::http::{Method, Uri};
use poem::web::FromRequest;
use poem::{Body, IntoResponse, Request};

#[tokio::test]
async fn poem_conforms_to_shared_excel_contract() {
    let runtime = runtime();
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/upload"))
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "fixture.csv")
        .header("x-request-id", "poem-upload")
        .body(Body::from_bytes(upload_fixture()));
    request.set_data(runtime.clone());
    let (request, mut body) = request.split();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut body)
        .await
        .expect("extract Poem upload");
    verify_upload(upload.into_rows())
        .await
        .expect("parse Poem upload");

    let response = ExcelResponse::prepare(
        download_rows(),
        Format::Xlsx,
        "conformance.xlsx",
        "data",
        runtime.context("poem-download"),
    )
    .await
    .expect("prepare Poem download")
    .into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .into_bytes()
        .await
        .expect("collect Poem body");
    verify_download(&ResponseSnapshot {
        status,
        content_type,
        content_disposition,
        body,
    });
}

fn header(headers: &poem::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
