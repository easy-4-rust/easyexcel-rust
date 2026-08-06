//! Axum 原生 `ExcelRequest<T>` / `ExcelResponse<T>` 示例。

use std::net::SocketAddr;

use axum::Router;
use axum::extract::State;
use axum::routing::{get, post};
use chrono::NaiveDateTime;
use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_axum::{
    ExcelRejection, ExcelRequest, ExcelResponse, ExcelWebPolicy, ExcelWebRuntime,
};
use tracing::info;

#[derive(Debug, Clone, ExcelRow)]
struct DownloadData {
    #[excel(name = "字符串标题", index = 0)]
    string: String,
    #[excel(name = "日期标题", index = 1)]
    date: NaiveDateTime,
    #[excel(name = "数字标题", index = 2)]
    double_data: f64,
}

#[derive(Debug, Clone, ExcelRow)]
struct UploadData {
    #[excel(index = 0)]
    string: String,
    #[excel(index = 1)]
    date: NaiveDateTime,
    #[excel(index = 2)]
    double_data: f64,
}

fn sample_download_rows() -> impl Iterator<Item = DownloadData> {
    let date = NaiveDateTime::parse_from_str("2020-01-01 00:00:00", "%Y-%m-%d %H:%M:%S")
        .expect("valid demo date");
    (0..10).map(move |_| DownloadData {
        string: "字符串0".to_owned(),
        date,
        double_data: 0.56,
    })
}

async fn download(
    State(runtime): State<ExcelWebRuntime>,
) -> Result<ExcelResponse<DownloadData>, ExcelRejection> {
    ExcelResponse::prepare(
        sample_download_rows(),
        Format::Xlsx,
        "测试.xlsx",
        "模板",
        runtime.generated_context(),
    )
    .await
}

async fn upload(request: ExcelRequest<UploadData>) -> Result<String, ExcelRejection> {
    let request_id = request.request_id().to_string();
    let mut rows = request.into_rows();
    let mut row_count = 0_u64;
    while let Some(row) = rows.next_row().await {
        let row = row.map_err(|error| ExcelRejection::new(error, &request_id))?;
        info!(?row, "解析到上传数据");
        row_count += 1;
    }
    Ok(format!("success: {row_count} rows"))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let policy = ExcelWebPolicy::new(ResourceLimits::default())
        .with_max_concurrent_tasks(4)
        .with_row_channel_capacity(32);
    let runtime = ExcelWebRuntime::new(policy);
    let app = Router::new()
        .route("/download", get(download))
        .route("/upload", post(upload))
        .with_state(runtime);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    info!("Axum EasyExcel 示例监听 http://{address}");
    let listener = tokio::net::TcpListener::bind(address).await.expect("bind");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("serve");
}

async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};
    let mut terminate = signal(SignalKind::terminate()).expect("SIGTERM handler");
    let mut interrupt = signal(SignalKind::interrupt()).expect("SIGINT handler");
    tokio::select! {
        _ = terminate.recv() => {}
        _ = interrupt.recv() => {}
    }
}
