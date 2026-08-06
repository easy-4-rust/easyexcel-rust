//! Rocket 原生流式 `EasyExcel` 示例。

#[macro_use]
extern crate rocket;

use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_rocket::{
    ExcelRequest, ExcelResponse, ExcelRocketError, ExcelWebPolicy, ExcelWebRuntime,
};
use rocket::State;

#[derive(Debug, Clone, ExcelRow)]
struct DemoRow {
    #[excel(name = "Name", index = 0)]
    name: String,
    #[excel(name = "Value", index = 1)]
    value: i64,
}

fn rows() -> impl Iterator<Item = DemoRow> {
    (0..10).map(|value| DemoRow {
        name: format!("row-{value}"),
        value,
    })
}

#[get("/download")]
async fn download(
    runtime: &State<ExcelWebRuntime>,
) -> Result<ExcelResponse<DemoRow>, ExcelRocketError> {
    ExcelResponse::prepare(
        rows(),
        Format::Xlsx,
        "rocket-example.xlsx",
        "Data",
        runtime.generated_context(),
    )
    .await
}

#[post("/upload", data = "<request>")]
async fn upload(request: ExcelRequest<DemoRow>) -> Result<String, ExcelRocketError> {
    let request_id = request.request_id().to_string();
    let mut rows = request.into_rows();
    let mut count = 0_u64;
    while let Some(row) = rows.next_row().await {
        row.map_err(|error| ExcelRocketError::new(error, &request_id))?;
        count += 1;
    }
    Ok(format!("success: {count} rows"))
}

#[launch]
fn rocket() -> _ {
    let runtime = ExcelWebRuntime::new(
        ExcelWebPolicy::new(ResourceLimits::default()).with_max_concurrent_tasks(4),
    );
    rocket::build()
        .manage(runtime)
        .mount("/", routes![download, upload])
}
