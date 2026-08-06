//! Poem 原生流式 `EasyExcel` 示例。

use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_poem::{
    ExcelPoemError, ExcelRequest, ExcelResponse, ExcelWebPolicy, ExcelWebRuntime,
};
use poem::endpoint::make_sync;
use poem::listener::TcpListener;
use poem::middleware::AddData;
use poem::web::Data;
use poem::{EndpointExt, Route, Server, get, handler, post};

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

#[handler]
async fn download(
    Data(runtime): Data<&ExcelWebRuntime>,
) -> Result<ExcelResponse<DemoRow>, ExcelPoemError> {
    ExcelResponse::prepare(
        rows(),
        Format::Xlsx,
        "poem-example.xlsx",
        "Data",
        runtime.generated_context(),
    )
    .await
}

#[handler]
async fn upload(request: ExcelRequest<DemoRow>) -> poem::Result<String> {
    let request_id = request.request_id().to_string();
    let mut rows = request.into_rows();
    let mut count = 0_u64;
    while let Some(row) = rows.next_row().await {
        row.map_err(|error| poem::Error::from(ExcelPoemError::new(error, &request_id)))?;
        count += 1;
    }
    Ok(format!("success: {count} rows"))
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    tracing_subscriber::fmt::init();
    let runtime = ExcelWebRuntime::new(
        ExcelWebPolicy::new(ResourceLimits::default()).with_max_concurrent_tasks(4),
    );
    let app = Route::new()
        .at("/download", get(download))
        .at("/upload", post(upload))
        .at("/health", get(make_sync(|_| "ok")))
        .with(AddData::new(runtime));
    Server::new(TcpListener::bind("127.0.0.1:8083"))
        .run(app)
        .await
}
