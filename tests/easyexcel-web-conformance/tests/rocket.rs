//! Rocket 对共享 Web 契约的实现验证。

use easyexcel::io::Format;
use easyexcel_rocket::{ExcelRequest, ExcelResponse, ExcelRocketError, ExcelWebRuntime};
use easyexcel_web_conformance::{
    ConformanceRow, ResponseSnapshot, download_rows, runtime, upload_fixture, verify_download,
    verify_upload,
};
use rocket::http::{ContentType, Header, Status};
use rocket::local::asynchronous::Client;
use rocket::{State, get, post, routes};

#[post("/upload", data = "<request>")]
async fn upload(request: ExcelRequest<ConformanceRow>) -> Result<&'static str, ExcelRocketError> {
    let request_id = request.request_id().to_string();
    verify_upload(request.into_rows())
        .await
        .map_err(|error| ExcelRocketError::new(error, request_id))?;
    Ok("ok")
}

#[get("/download")]
async fn download(
    runtime: &State<ExcelWebRuntime>,
) -> Result<ExcelResponse<ConformanceRow>, ExcelRocketError> {
    ExcelResponse::prepare(
        download_rows(),
        Format::Xlsx,
        "conformance.xlsx",
        "data",
        runtime.context("rocket-download"),
    )
    .await
}

#[rocket::async_test]
async fn rocket_conforms_to_shared_excel_contract() {
    let rocket = rocket::build()
        .manage(runtime())
        .mount("/", routes![upload, download]);
    let client = Client::tracked(rocket).await.expect("create Rocket client");
    let upload_response = client
        .post("/upload")
        .header(ContentType::CSV)
        .header(Header::new("x-excel-file-name", "fixture.csv"))
        .header(Header::new("x-request-id", "rocket-upload"))
        .body(upload_fixture())
        .dispatch()
        .await;
    assert_eq!(upload_response.status(), Status::Ok);

    let response = client.get("/download").dispatch().await;
    let status = response.status().code;
    let content_type = response
        .headers()
        .get_one("content-type")
        .unwrap_or_default()
        .to_string();
    let content_disposition = response
        .headers()
        .get_one("content-disposition")
        .unwrap_or_default()
        .to_string();
    let body = response
        .into_bytes()
        .await
        .expect("collect Rocket body")
        .into();
    verify_download(&ResponseSnapshot {
        status,
        content_type,
        content_disposition,
        body,
    });
}
