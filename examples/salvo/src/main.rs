//! Salvo 原生流式 `EasyExcel` 示例。

use easyexcel::ExcelRow;
use easyexcel::io::{Format, ResourceLimits};
use easyexcel_salvo::{
    ExcelRequest, ExcelResponse, ExcelSalvoError, ExcelWebPolicy, ExcelWebRuntime,
};
use salvo::Extractible;
use salvo::prelude::*;

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

#[derive(Clone)]
struct RuntimeHoop(ExcelWebRuntime);

#[async_trait]
impl Handler for RuntimeHoop {
    async fn handle(
        &self,
        request: &mut Request,
        depot: &mut Depot,
        response: &mut Response,
        control: &mut FlowCtrl,
    ) {
        request.extensions_mut().insert(self.0.clone());
        control.call_next(request, depot, response).await;
    }
}

#[handler]
async fn download(request: &mut Request, depot: &mut Depot, response: &mut Response) {
    let runtime = request
        .extensions()
        .get::<ExcelWebRuntime>()
        .expect("runtime attached")
        .clone();
    match ExcelResponse::prepare(
        rows(),
        Format::Xlsx,
        "salvo-example.xlsx",
        "Data",
        runtime.generated_context(),
    )
    .await
    {
        Ok(excel_response) => excel_response.write(request, depot, response).await,
        Err(error) => error.write(request, depot, response).await,
    }
}

#[handler]
async fn upload(request: &mut Request, depot: &mut Depot, response: &mut Response) {
    match ExcelRequest::<DemoRow>::extract(request).await {
        Ok(excel_request) => {
            let request_id = excel_request.request_id().to_string();
            let mut rows = excel_request.into_rows();
            let mut count = 0_u64;
            while let Some(row) = rows.next_row().await {
                if let Err(error) = row {
                    ExcelSalvoError::new(error, &request_id)
                        .write(request, depot, response)
                        .await;
                    return;
                }
                count += 1;
            }
            response.render(format!("success: {count} rows"));
        }
        Err(error) => error.write(request, depot, response).await,
    }
}

#[tokio::main]
async fn main() {
    let runtime = ExcelWebRuntime::new(
        ExcelWebPolicy::new(ResourceLimits::default()).with_max_concurrent_tasks(4),
    );
    let router = Router::new()
        .hoop(RuntimeHoop(runtime))
        .push(Router::with_path("download").get(download))
        .push(Router::with_path("upload").post(upload));
    let acceptor = TcpListener::new("127.0.0.1:8084").bind().await;
    Server::new(acceptor).serve(router).await;
}
