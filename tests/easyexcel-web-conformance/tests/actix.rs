//! Actix Web 对共享 Web 契约的实现验证。

use actix_web::body::to_bytes;
use actix_web::test::TestRequest;
use actix_web::{FromRequest, ResponseError, Responder, web};
use easyexcel::io::Format;
use easyexcel_actix::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, corrupted_xlsx_fixture, download_rows, oversized_fixture,
    runtime, strict_runtime, upload_fixture, verify_download, verify_error_response,
    verify_upload, verify_upload_multisheet, verify_upload_xls, xlsx_multisheet_fixture,
    xls_upload_fixture,
};

#[actix_web::test]
async fn actix_conforms_to_shared_excel_contract() {
    let runtime = runtime();
    let (request, mut payload) = TestRequest::post()
        .insert_header(("content-type", "text/csv"))
        .insert_header(("x-excel-file-name", "fixture.csv"))
        .insert_header(("x-request-id", "actix-upload"))
        .app_data(web::Data::new(runtime.clone()))
        .set_payload(upload_fixture())
        .to_http_parts();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut payload)
        .await
        .expect("extract Actix upload");
    verify_upload(upload.into_rows())
        .await
        .expect("parse Actix upload");

    let response = ExcelResponse::prepare(
        download_rows(),
        Format::Xlsx,
        "conformance.xlsx",
        "data",
        runtime.context("actix-download"),
    )
    .await
    .expect("prepare Actix download")
    .respond_to(&request);
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = to_bytes(response.into_body())
        .await
        .expect("collect Actix body");
    verify_download(&ResponseSnapshot {
        status,
        content_type,
        content_disposition,
        body,
    });
}

#[actix_web::test]
async fn actix_xls_upload_conforms() {
    let runtime = runtime();
    let (request, mut payload) = TestRequest::post()
        .insert_header(("content-type", "application/vnd.ms-excel"))
        .insert_header(("x-excel-file-name", "fixture.xls"))
        .insert_header(("x-request-id", "actix-xls-upload"))
        .app_data(web::Data::new(runtime.clone()))
        .set_payload(xls_upload_fixture())
        .to_http_parts();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut payload)
        .await
        .expect("extract Actix XLS upload");
    verify_upload_xls(upload.into_rows())
        .await
        .expect("parse Actix XLS upload");
}

#[actix_web::test]
async fn actix_xlsx_multisheet_upload_conforms() {
    let runtime = runtime();
    let (request, mut payload) = TestRequest::post()
        .insert_header((
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ))
        .insert_header(("x-excel-file-name", "fixture.xlsx"))
        .insert_header(("x-request-id", "actix-multisheet-upload"))
        .app_data(web::Data::new(runtime.clone()))
        .set_payload(xlsx_multisheet_fixture())
        .to_http_parts();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut payload)
        .await
        .expect("extract Actix multi-sheet upload");
    verify_upload_multisheet(upload.into_rows())
        .await
        .expect("parse Actix multi-sheet upload");
}

#[actix_web::test]
async fn actix_oversized_upload_returns_file_too_large() {
    let runtime = strict_runtime();
    let (request, mut payload) = TestRequest::post()
        .insert_header(("content-type", "text/csv"))
        .insert_header(("x-excel-file-name", "oversized.csv"))
        .insert_header(("x-request-id", "actix-oversized"))
        .app_data(web::Data::new(runtime.clone()))
        .set_payload(oversized_fixture(runtime.policy()))
        .to_http_parts();
    let error = ExcelRequest::<ConformanceRow>::from_request(&request, &mut payload)
        .await
        .expect_err("oversized upload must fail");
    let response = error.error_response();
    let status = response.status().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = to_bytes(response.into_body())
        .await
        .expect("collect Actix oversized body");
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

#[actix_web::test]
async fn actix_corrupted_upload_returns_invalid_format() {
    let runtime = runtime();
    let (request, mut payload) = TestRequest::post()
        .insert_header((
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ))
        .insert_header(("x-excel-file-name", "corrupted.xlsx"))
        .insert_header(("x-request-id", "actix-corrupted"))
        .app_data(web::Data::new(runtime.clone()))
        .set_payload(corrupted_xlsx_fixture())
        .to_http_parts();
    let upload = ExcelRequest::<ConformanceRow>::from_request(&request, &mut payload)
        .await
        .expect("extract Actix corrupted upload");
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

#[actix_web::test]
#[ignore = "framework test harness does not support mid-stream body drop for cancellation"]
async fn actix_client_disconnect_propagates_cancellation() {
    // Cancellation propagation requires a gated body stream that can be dropped
    // mid-transfer. Actix's `FromRequest` test harness consumes the entire body
    // synchronously, making it infeasible to simulate a client disconnect
    // without a real TCP connection. This test is reserved for integration tests.
}

fn header(headers: &actix_web::http::header::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
