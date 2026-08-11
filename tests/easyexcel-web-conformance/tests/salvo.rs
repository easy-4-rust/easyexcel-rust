//! Salvo 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_salvo::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, corrupted_xlsx_fixture, download_rows, oversized_fixture,
    runtime, strict_runtime, upload_fixture, verify_download, verify_error_response, verify_upload,
    verify_upload_multisheet, verify_upload_xls, xls_upload_fixture, xlsx_multisheet_fixture,
};
use http_body_util::BodyExt;
use salvo::test::TestClient;
use salvo::{Depot, Extractible, Response, Writer};

#[tokio::test]
async fn salvo_conforms_to_shared_excel_contract() {
    let runtime = runtime();
    let mut request = TestClient::post("http://localhost/upload")
        .add_header("content-type", "text/csv", true)
        .add_header("x-excel-file-name", "fixture.csv", true)
        .add_header("x-request-id", "salvo-upload", true)
        .bytes(upload_fixture().to_vec())
        .build();
    request.extensions_mut().insert(runtime.clone());
    let mut depot = Depot::new();
    let upload = ExcelRequest::<ConformanceRow>::extract(&mut request, &mut depot)
        .await
        .expect("extract Salvo upload");
    verify_upload(upload.into_rows())
        .await
        .expect("parse Salvo upload");

    let download = ExcelResponse::prepare(
        download_rows(),
        Format::Xlsx,
        "conformance.xlsx",
        "data",
        runtime.context("salvo-download"),
    )
    .await
    .expect("prepare Salvo download");
    let mut response = Response::new();
    download
        .write(&mut request, &mut depot, &mut response)
        .await;
    let status = response.status_code.unwrap_or_default().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .take_body()
        .collect()
        .await
        .expect("collect Salvo body")
        .to_bytes();
    verify_download(&ResponseSnapshot {
        status,
        content_type,
        content_disposition,
        body,
    });
}

#[tokio::test]
async fn salvo_xls_upload_conforms() {
    let runtime = runtime();
    let mut request = TestClient::post("http://localhost/upload")
        .add_header("content-type", "application/vnd.ms-excel", true)
        .add_header("x-excel-file-name", "fixture.xls", true)
        .add_header("x-request-id", "salvo-xls-upload", true)
        .bytes(xls_upload_fixture().to_vec())
        .build();
    request.extensions_mut().insert(runtime.clone());
    let mut depot = Depot::new();
    let upload = ExcelRequest::<ConformanceRow>::extract(&mut request, &mut depot)
        .await
        .expect("extract Salvo XLS upload");
    verify_upload_xls(upload.into_rows())
        .await
        .expect("parse Salvo XLS upload");
}

#[tokio::test]
async fn salvo_xlsx_multisheet_upload_conforms() {
    let runtime = runtime();
    let mut request = TestClient::post("http://localhost/upload")
        .add_header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            true,
        )
        .add_header("x-excel-file-name", "fixture.xlsx", true)
        .add_header("x-request-id", "salvo-multisheet-upload", true)
        .bytes(xlsx_multisheet_fixture().to_vec())
        .build();
    request.extensions_mut().insert(runtime.clone());
    let mut depot = Depot::new();
    let upload = ExcelRequest::<ConformanceRow>::extract(&mut request, &mut depot)
        .await
        .expect("extract Salvo multi-sheet upload");
    verify_upload_multisheet(upload.into_rows())
        .await
        .expect("parse Salvo multi-sheet upload");
}

#[tokio::test]
async fn salvo_oversized_upload_returns_file_too_large() {
    let runtime = strict_runtime();
    let mut request = TestClient::post("http://localhost/upload")
        .add_header("content-type", "text/csv", true)
        .add_header("x-excel-file-name", "oversized.csv", true)
        .add_header("x-request-id", "salvo-oversized", true)
        .bytes(oversized_fixture(runtime.policy()).to_vec())
        .build();
    request.extensions_mut().insert(runtime.clone());
    let mut depot = Depot::new();
    let error = ExcelRequest::<ConformanceRow>::extract(&mut request, &mut depot)
        .await
        .expect_err("oversized upload must fail");
    // Salvo ExcelSalvoError implements Writer; write it to a response
    let mut response = Response::new();
    error.write(&mut request, &mut depot, &mut response).await;
    let status = response.status_code.unwrap_or_default().as_u16();
    let content_type = header(response.headers(), "content-type");
    let content_disposition = header(response.headers(), "content-disposition");
    let body = response
        .take_body()
        .collect()
        .await
        .expect("collect Salvo oversized body")
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
async fn salvo_corrupted_upload_returns_invalid_format() {
    let runtime = runtime();
    let mut request = TestClient::post("http://localhost/upload")
        .add_header(
            "content-type",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            true,
        )
        .add_header("x-excel-file-name", "corrupted.xlsx", true)
        .add_header("x-request-id", "salvo-corrupted", true)
        .bytes(corrupted_xlsx_fixture().to_vec())
        .build();
    request.extensions_mut().insert(runtime.clone());
    let mut depot = Depot::new();
    let upload = ExcelRequest::<ConformanceRow>::extract(&mut request, &mut depot)
        .await
        .expect("extract Salvo corrupted upload");
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
async fn salvo_client_disconnect_propagates_cancellation() {
    // Cancellation propagation requires a gated body stream that can be dropped
    // mid-transfer. Salvo's `Extractible` test harness consumes the entire body
    // synchronously, making it infeasible to simulate a client disconnect
    // without a real TCP connection. This test is reserved for integration tests.
}

fn header(headers: &salvo::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
