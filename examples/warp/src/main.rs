//! Warp 原生流式 `EasyExcel` 示例。

use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_warp::{
    ExcelRequest, ExcelResponse, ExcelWarpRejection, ExcelWebPolicy, ExcelWebRuntime,
    excel_request, recover_excel_rejection,
};
use warp::Filter;

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

#[tokio::main]
async fn main() {
    let runtime = ExcelWebRuntime::new(
        ExcelWebPolicy::new(ResourceLimits::default()).with_max_concurrent_tasks(4),
    );
    let download_runtime = runtime.clone();
    let download = warp::path("download").and(warp::get()).and_then(move || {
        let runtime = download_runtime.clone();
        async move {
            ExcelResponse::prepare(
                rows(),
                Format::Xlsx,
                "warp-example.xlsx",
                "Data",
                runtime.generated_context(),
            )
            .await
        }
    });
    let upload = warp::path("upload")
        .and(warp::post())
        .and(excel_request::<DemoRow>(runtime))
        .and_then(upload_rows);
    warp::serve(download.or(upload).recover(recover_excel_rejection))
        .run(([127, 0, 0, 1], 8085))
        .await;
}

async fn upload_rows(request: ExcelRequest<DemoRow>) -> Result<String, warp::Rejection> {
    let request_id = request.request_id().to_string();
    let mut rows = request.into_rows();
    let mut count = 0_u64;
    while let Some(row) = rows.next_row().await {
        row.map_err(|error| warp::reject::custom(ExcelWarpRejection::new(error, &request_id)))?;
        count += 1;
    }
    Ok(format!("success: {count} rows"))
}
