//! Hyper 原生流式 `EasyExcel` 示例。

use std::convert::Infallible;
use std::io;

use bytes::Bytes;
use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_hyper::{
    ExcelHyperError, ExcelRequest, ExcelResponse, ExcelWebPolicy, ExcelWebRuntime, ResponseBody,
};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

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

async fn handle(
    request: Request<Incoming>,
    runtime: ExcelWebRuntime,
) -> Result<Response<ResponseBody>, Infallible> {
    let response = match (request.method().as_str(), request.uri().path()) {
        ("GET", "/download") => ExcelResponse::prepare(
            rows(),
            Format::Xlsx,
            "hyper-example.xlsx",
            "Data",
            runtime.generated_context(),
        )
        .await
        .map_or_else(ExcelHyperError::into_response, ExcelResponse::into_response),
        ("POST", "/upload") => match ExcelRequest::<DemoRow>::from_request(request, &runtime).await
        {
            Ok(excel_request) => {
                let request_id = excel_request.request_id().to_string();
                let mut excel_rows = excel_request.into_rows();
                let mut count = 0_u64;
                let mut error = None;
                while let Some(row) = excel_rows.next_row().await {
                    match row {
                        Ok(_row) => count += 1,
                        Err(row_error) => {
                            error = Some(ExcelHyperError::new(row_error, &request_id));
                            break;
                        }
                    }
                }
                error.map_or_else(
                    || text_response(format!("success: {count} rows")),
                    ExcelHyperError::into_response,
                )
            }
            Err(error) => error.into_response(),
        },
        _ => {
            let mut response = text_response("not found".to_string());
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
        }
    };
    Ok(response)
}

fn text_response(value: String) -> Response<ResponseBody> {
    let body = Full::new(Bytes::from(value))
        .map_err(|never| match never {})
        .boxed();
    Response::new(body)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    tracing_subscriber::fmt::init();
    let runtime = ExcelWebRuntime::new(
        ExcelWebPolicy::new(ResourceLimits::default()).with_max_concurrent_tasks(4),
    );
    let listener = TcpListener::bind(("127.0.0.1", 8082)).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let runtime = runtime.clone();
        tokio::spawn(async move {
            let service = service_fn(move |request| handle(request, runtime.clone()));
            if let Err(error) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service)
                .await
            {
                tracing::warn!(%error, "Hyper connection failed");
            }
        });
    }
}
