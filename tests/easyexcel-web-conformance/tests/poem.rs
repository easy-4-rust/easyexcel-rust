//! Poem 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_poem::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, corrupted_xlsx_fixture, download_rows, oversized_fixture,
    runtime, strict_runtime, upload_fixture, verify_download, verify_error_response,
    verify_upload, verify_upload_multisheet, verify_upload_xls, xlsx_multisheet_fixture,
    xls_upload_fixture,
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

#[tokio::test]
async fn poem_xls_upload_conforms() {
    let runtime = runtime();
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/upload"))
        .header("content-type", "application/vnd.ms-excel")
        .header("x-excel-file-name", "fixture.xls")
        .header("x-request-id", "poem-xls-upload")
        .body(Body::from_bytes(xls_upload_fixture()));
    request.set_data(runtime.clone());
    let (request, mut body) = request.split();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut body)
        .await
        .expect("extract Poem XLS upload");
    verify_upload_xls(upload.into_rows())
        .await
        .expect("parse Poem XLS upload");
}

#[tokio::test]
async fn poem_xlsx_multisheet_upload_conforms() {
    let runtime = runtime();
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/upload"))
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "fixture.xlsx")
        .header("x-request-id", "poem-multisheet-upload")
        .body(Body::from_bytes(xlsx_multisheet_fixture()));
    request.set_data(runtime.clone());
    let (request, mut body) = request.split();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut body)
        .await
        .expect("extract Poem multi-sheet upload");
    verify_upload_multisheet(upload.into_rows())
        .await
        .expect("parse Poem multi-sheet upload");
}

#[tokio::test]
async fn poem_oversized_upload_returns_file_too_large() {
    let runtime = strict_runtime();
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/upload"))
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "oversized.csv")
        .header("x-request-id", "poem-oversized")
        .body(Body::from_bytes(oversized_fixture(runtime.policy())));
    request.set_data(runtime.clone());
    let (request, mut body) = request.split();
    let error = ExcelRequest::<ConformanceRow>::from_request(&request, &mut body)
        .await
        .expect_err("oversized upload must fail");
    let response = error.into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .into_bytes()
        .await
        .expect("collect Poem oversized body");
    verify_error_response(
        &ResponseSnapshot {
            status,
            content_type,
            content_disposition,
            body,
        },
        "FILE_TOO_LARGE",
    );
}

#[tokio::test]
async fn poem_corrupted_upload_returns_invalid_format() {
    let runtime = runtime();
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/upload"))
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "corrupted.xlsx")
        .header("x-request-id", "poem-corrupted")
        .body(Body::from_bytes(corrupted_xlsx_fixture()));
    request.set_data(runtime.clone());
    let (request, mut body) = request.split();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut body)
        .await
        .expect("extract Poem corrupted upload");
    let mut rows = upload.into_rows();
    let error = rows
        .next_row()
        .await
        .expect("corrupted parse must yield result")
        .expect_err("corrupted parse must fail");
    assert!(
        error.code() == easyexcel_web::ExcelWebErrorCode::InvalidFormat
            || error.code() == easyexcel_web::ExcelWebErrorCode::RowConversionFailed
            || error.code() == easyexcel_web::ExcelWebErrorCode::Internal,
        "unexpected error code: {:?}",
        error.code()
    );
}

#[tokio::test]
#[ignore = "framework test harness does not support mid-stream body drop for cancellation"]
async fn poem_client_disconnect_propagates_cancellation() {
    // Cancellation propagation requires a gated body stream that can be dropped
    // mid-transfer. Poem's `FromRequest` test harness consumes the entire body
    // synchronously, making it infeasible to simulate a client disconnect
    // without a real TCP connection. This test is reserved for integration tests.
}

fn header(headers: &poem::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
