//! Salvo 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_salvo::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, download_rows, runtime, upload_fixture, verify_download,
    verify_upload,
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
    let upload = ExcelRequest::<ConformanceRow>::extract(&mut request)
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

fn header(headers: &salvo::http::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
