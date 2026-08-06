//! Actix Web 原生 `ExcelRequest<T>` / `ExcelResponse<T>` 示例。

use actix_web::{App, HttpServer, web};
use chrono::NaiveDateTime;
use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_actix::{
    ExcelActixError, ExcelRequest, ExcelResponse, ExcelWebPolicy, ExcelWebRuntime,
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
    runtime: web::Data<ExcelWebRuntime>,
) -> Result<ExcelResponse<DownloadData>, ExcelActixError> {
    ExcelResponse::prepare(
        sample_download_rows(),
        Format::Xlsx,
        "测试.xlsx",
        "模板",
        runtime.generated_context(),
    )
    .await
}

async fn upload(request: ExcelRequest<UploadData>) -> Result<String, ExcelActixError> {
    let request_id = request.request_id().to_string();
    let mut rows = request.into_rows();
    let mut row_count = 0_u64;
    while let Some(row) = rows.next_row().await {
        let row = row.map_err(|error| ExcelActixError::new(error, &request_id))?;
        info!(?row, "解析到上传数据");
        row_count += 1;
    }
    Ok(format!("success: {row_count} rows"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8081);
    let policy = ExcelWebPolicy::new(ResourceLimits::default())
        .with_max_concurrent_tasks(4)
        .with_row_channel_capacity(32);
    let runtime = web::Data::new(ExcelWebRuntime::new(policy));
    info!("Actix EasyExcel 示例监听 http://127.0.0.1:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(runtime.clone())
            .route("/download", web::get().to(download))
            .route("/upload", web::post().to(upload))
    })
    .bind(("127.0.0.1", port))?
    .run()
    .await
}
