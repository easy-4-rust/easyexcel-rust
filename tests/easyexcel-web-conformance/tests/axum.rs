//! Axum 对共享 Web 契约的实现验证。

use axum::body::Body;
use axum::extract::FromRequest;
use axum::response::IntoResponse;
use easyexcel::io::Format;
use easyexcel_axum::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, corrupted_xlsx_fixture, download_rows, oversized_fixture,
    runtime, strict_runtime, upload_fixture, verify_download, verify_error_response,
    verify_upload, verify_upload_multisheet, verify_upload_xls, xlsx_multisheet_fixture,
    xls_upload_fixture,
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

#[tokio::test]
async fn axum_xls_upload_conforms() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header("content-type", "application/vnd.ms-excel")
        .header("x-excel-file-name", "fixture.xls")
        .header("x-request-id", "axum-xls-upload")
        .body(Body::from(xls_upload_fixture()))
        .expect("build Axum XLS request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Axum XLS upload");
    verify_upload_xls(upload.into_rows())
        .await
        .expect("parse Axum XLS upload");
}

#[tokio::test]
async fn axum_xlsx_multisheet_upload_conforms() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "fixture.xlsx")
        .header("x-request-id", "axum-multisheet-upload")
        .body(Body::from(xlsx_multisheet_fixture()))
        .expect("build Axum multi-sheet request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Axum multi-sheet upload");
    verify_upload_multisheet(upload.into_rows())
        .await
        .expect("parse Axum multi-sheet upload");
}

#[tokio::test]
async fn axum_oversized_upload_returns_file_too_large() {
    let runtime = strict_runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "oversized.csv")
        .header("x-request-id", "axum-oversized")
        .body(Body::from(oversized_fixture(runtime.policy())))
        .expect("build Axum oversized request");
    let rejection = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect_err("oversized upload must fail");
    let response = rejection.into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect Axum oversized body")
        .to_bytes();
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
async fn axum_corrupted_upload_returns_invalid_format() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "corrupted.xlsx")
        .header("x-request-id", "axum-corrupted")
        .body(Body::from(corrupted_xlsx_fixture()))
        .expect("build Axum corrupted request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Axum corrupted upload");
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
async fn axum_client_disconnect_propagates_cancellation() {
    // Cancellation propagation requires a gated body stream that can be dropped
    // mid-transfer. Axum's `FromRequest` test harness consumes the entire body
    // synchronously, making it infeasible to simulate a client disconnect
    // without a real TCP connection. This test is reserved for integration tests.
}

fn header(headers: &http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
