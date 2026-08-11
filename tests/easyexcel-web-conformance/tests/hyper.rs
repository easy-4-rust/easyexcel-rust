//! Hyper 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_hyper::{ExcelHyperError, ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, corrupted_xlsx_fixture, download_rows, oversized_fixture,
    runtime, strict_runtime, upload_fixture, verify_download, verify_error_response, verify_upload,
    verify_upload_multisheet, verify_upload_xls, xls_upload_fixture, xlsx_multisheet_fixture,
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

#[tokio::test]
async fn hyper_xls_upload_conforms() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header("content-type", "application/vnd.ms-excel")
        .header("x-excel-file-name", "fixture.xls")
        .header("x-request-id", "hyper-xls-upload")
        .body(Full::new(xls_upload_fixture()))
        .expect("build Hyper XLS request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Hyper XLS upload");
    verify_upload_xls(upload.into_rows())
        .await
        .expect("parse Hyper XLS upload");
}

#[tokio::test]
async fn hyper_xlsx_multisheet_upload_conforms() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "fixture.xlsx")
        .header("x-request-id", "hyper-multisheet-upload")
        .body(Full::new(xlsx_multisheet_fixture()))
        .expect("build Hyper multi-sheet request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Hyper multi-sheet upload");
    verify_upload_multisheet(upload.into_rows())
        .await
        .expect("parse Hyper multi-sheet upload");
}

#[tokio::test]
async fn hyper_oversized_upload_returns_file_too_large() {
    let runtime = strict_runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "oversized.csv")
        .header("x-request-id", "hyper-oversized")
        .body(Full::new(oversized_fixture(runtime.policy())))
        .expect("build Hyper oversized request");
    let error: ExcelHyperError = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect_err("oversized upload must fail");
    let response = error.into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect Hyper oversized body")
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
async fn hyper_corrupted_upload_returns_invalid_format() {
    let runtime = runtime();
    let request = Request::builder()
        .method("POST")
        .uri("/upload")
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "corrupted.xlsx")
        .header("x-request-id", "hyper-corrupted")
        .body(Full::new(corrupted_xlsx_fixture()))
        .expect("build Hyper corrupted request");
    let upload = ExcelRequest::<ConformanceRow>::from_request(request, &runtime)
        .await
        .expect("extract Hyper corrupted upload");
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
async fn hyper_client_disconnect_propagates_cancellation() {
    // Cancellation propagation requires a gated body stream that can be dropped
    // mid-transfer. Hyper's `FromRequest` test harness consumes the entire body
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
