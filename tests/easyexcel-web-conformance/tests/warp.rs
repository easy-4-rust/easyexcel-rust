//! Warp 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_warp::{ExcelRequest, ExcelResponse, excel_request};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, corrupted_xlsx_fixture, download_rows, oversized_fixture,
    runtime, strict_runtime, upload_fixture, verify_download, verify_error_response,
    verify_upload, verify_upload_multisheet, verify_upload_xls, xlsx_multisheet_fixture,
    xls_upload_fixture,
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

#[tokio::test]
async fn warp_xls_upload_conforms() {
    let runtime = runtime();
    let filter = excel_request::<ConformanceRow>(runtime.clone());
    let upload: ExcelRequest<ConformanceRow> = warp::test::request()
        .method("POST")
        .header("content-type", "application/vnd.ms-excel")
        .header("x-excel-file-name", "fixture.xls")
        .header("x-request-id", "warp-xls-upload")
        .body(xls_upload_fixture())
        .filter(&filter)
        .await
        .expect("extract Warp XLS upload");
    verify_upload_xls(upload.into_rows())
        .await
        .expect("parse Warp XLS upload");
}

#[tokio::test]
async fn warp_xlsx_multisheet_upload_conforms() {
    let runtime = runtime();
    let filter = excel_request::<ConformanceRow>(runtime.clone());
    let upload: ExcelRequest<ConformanceRow> = warp::test::request()
        .method("POST")
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "fixture.xlsx")
        .header("x-request-id", "warp-multisheet-upload")
        .body(xlsx_multisheet_fixture())
        .filter(&filter)
        .await
        .expect("extract Warp multi-sheet upload");
    verify_upload_multisheet(upload.into_rows())
        .await
        .expect("parse Warp multi-sheet upload");
}

#[tokio::test]
async fn warp_oversized_upload_returns_file_too_large() {
    let runtime = strict_runtime();
    let filter = excel_request::<ConformanceRow>(runtime.clone());
    let rejection = warp::test::request()
        .method("POST")
        .header("content-type", "text/csv")
        .header("x-excel-file-name", "oversized.csv")
        .header("x-request-id", "warp-oversized")
        .body(oversized_fixture(runtime.policy()))
        .filter(&filter)
        .await
        .expect_err("oversized upload must fail");
    let response = easyexcel_warp::recover_excel_rejection(rejection)
        .await
        .expect("recover Warp rejection")
        .into_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .into_body()
        .collect()
        .await
        .expect("collect Warp oversized body")
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
async fn warp_corrupted_upload_returns_invalid_format() {
    let runtime = runtime();
    let filter = excel_request::<ConformanceRow>(runtime.clone());
    let upload: ExcelRequest<ConformanceRow> = warp::test::request()
        .method("POST")
        .header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        )
        .header("x-excel-file-name", "corrupted.xlsx")
        .header("x-request-id", "warp-corrupted")
        .body(corrupted_xlsx_fixture())
        .filter(&filter)
        .await
        .expect("extract Warp corrupted upload");
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
async fn warp_client_disconnect_propagates_cancellation() {
    // Cancellation propagation requires a gated body stream that can be dropped
    // mid-transfer. Warp's test harness consumes the entire body synchronously,
    // making it infeasible to simulate a client disconnect without a real TCP
    // connection. This test is reserved for integration tests.
}

fn header(headers: &http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
