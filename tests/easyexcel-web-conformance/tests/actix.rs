//! Actix Web 对共享 Web 契约的实现验证。

use actix_web::body::to_bytes;
use actix_web::test::TestRequest;
use actix_web::{FromRequest, Responder, web};
use easyexcel::io::Format;
use easyexcel_actix::{ExcelRequest, ExcelResponse};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, download_rows, runtime, upload_fixture, verify_download,
    verify_upload,
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

fn header(headers: &actix_web::http::header::HeaderMap, name: &str) -> String {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}
