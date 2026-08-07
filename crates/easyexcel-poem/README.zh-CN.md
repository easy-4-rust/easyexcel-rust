# easyexcel-poem

[English](README.md)

面向 Poem 的 EasyExcel 原生请求提取与响应适配器。

> 版本: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

`easyexcel-poem` 只负责把 Poem 传输类型桥接到 `easyexcel-web`。上传落盘、资源限制、行流背压、取消、超时、临时文件清理与稳定错误协议均由共享内核实现，避免各框架产生不同语义。

原生集成方式：`FromRequest` extractor and `IntoResponse` responder；运行时传递方式：`Data<&ExcelWebRuntime>` plus `AddData`。

## 一览

```text
HTTP 请求 -> easyexcel-poem -> easyexcel-web -> 类型化行 / 流式响应
```

## 架构

```mermaid
flowchart LR
    Request["Poem 请求"] --> Adapter["easyexcel-poem"]
    Adapter --> Import["easyexcel-web / ExcelImport"]
    Import --> Rows["ExcelRows<T> / 背压"]
    Rows --> Handler["业务 Handler"]
    Handler --> Export["ExcelExport<T>"]
    Export --> Response["Poem 响应"]
```

适配器不得重新实现 Excel 解析、写入或资源策略。业务行在有界通道中消费，下载通过异步文件流交给 Poem。

## 能力矩阵

| 能力 | 状态 | 实现 |
|:---|:---|:---|
| `ExcelRequest<T>` | 可用 | 原生 Poem 请求提取，返回类型化背压行流。 |
| `ExcelResponse<T>` | 可用 | 发送响应头前完成受控文件生成，再异步传输。 |
| 资源与并发 | 共享 | `ExcelWebPolicy` + `ExcelWebRuntime` |
| 错误协议 | 稳定 | `ExcelPoemError` + `ExcelProblemDetails` |
| TUI / HTML form | 范围外 | 由业务应用或 examples 提供。 |

## 安装

```toml
[dependencies]
easyexcel = "0.1.3"
easyexcel-poem = "0.1.3"
```

所有工作簿 API 仍通过 `easyexcel::...` 使用；只有 Poem 原生 extractor、responder 与错误类型来自本适配器。适配器依赖 `easyexcel`，门面反向重导出会形成循环依赖。两个 crate 必须保持同一发布线。

## 定义行模型

```rust
use easyexcel::ExcelRow;

#[derive(Debug, ExcelRow)]
struct ReportRow {
    #[excel(name = "Name")]
    name: String,

    #[excel(name = "Value", number_format = "0")]
    value: i64,
}

fn report_rows() -> impl Iterator<Item = ReportRow> {
    (0..10).map(|value| ReportRow {
        name: format!("row-{value}"),
        value,
    })
}
```

## 流式下载

```rust
use easyexcel::io::Format;
use easyexcel_poem::{ExcelPoemError, ExcelResponse, ExcelWebRuntime};
use poem::web::Data;
use poem::handler;

#[handler]
async fn download(
    Data(runtime): Data<&ExcelWebRuntime>,
) -> Result<ExcelResponse<ReportRow>, ExcelPoemError> {
    ExcelResponse::prepare(
        report_rows(),
        Format::Xlsx,
        "report.xlsx",
        "Data",
        runtime.generated_context(),
    ).await
}
```

`ExcelResponse::prepare` 在返回 Poem 响应前完成生成和限制校验；成功后响应体从临时文件异步读取，不把完整文件复制到内存。

## 背压上传

```rust
use easyexcel_poem::{ExcelPoemError, ExcelRequest};
use poem::handler;

#[handler]
async fn upload(request: ExcelRequest<ReportRow>) -> poem::Result<String> {
    let request_id = request.request_id().to_owned();
    let mut rows = request.into_rows();
    let mut count = 0_u64;
    while let Some(row) = rows.next_row().await {
        row.map_err(|error| poem::Error::from(
            ExcelPoemError::new(error, &request_id)
        ))?;
        count += 1;
    }
    Ok(format!("success: {count} rows"))
}
```

上传请求必须提供 `x-excel-file-name`、`Content-Disposition` 或可识别的 `Content-Type`。可选 `x-request-id` 会进入 tracing 与错误响应。

## 运行时接线

```rust
use poem::{EndpointExt, Route, get, post};
use poem::middleware::AddData;
use easyexcel_poem::{ExcelWebPolicy, ExcelWebRuntime};

let runtime = ExcelWebRuntime::new(ExcelWebPolicy::default());
let app = Route::new()
    .at("/download", get(download))
    .at("/upload", post(upload))
    .with(AddData::new(runtime));
```

应用应创建一个共享 `ExcelWebRuntime`，而不是为每个请求重新创建并发许可池。可通过 `ExcelWebPolicy` 统一设置文件字节数、行数、上传/处理超时、最大任务数、行通道容量和临时目录。

## 响应头与错误

- `Content-Type` 根据 XLSX、XLS 或 CSV 格式生成。
- `Content-Disposition` 使用 UTF-8 文件名编码并清理不安全名称。
- `Content-Length` 来自生成后文件大小。
- `ExcelPoemError` 把共享错误映射为框架原生 rejection/error/response。
- 诊断信息进入 tracing；响应体保持稳定问题详情，不泄露内部路径。

## 能力边界

- 流式上传表示请求体分块落盘后再解析，不表示 XLS/XLSX 容器可在未接收完整文件时随机访问。
- 流式下载在生成成功后开始传输，避免客户端收到部分有效工作簿。
- 适配器不承载业务校验、鉴权或持久化；这些职责属于应用 handler/middleware。
- 完整可运行服务位于 `examples/poem`，共享一致性断言位于 `tests/easyexcel-web-conformance`。

## 依赖关系

```mermaid
flowchart TB
    Framework["Poem"] --> Adapter["easyexcel-poem"]
    Adapter --> Web["easyexcel-web"]
    Web --> Facade["easyexcel"]
    Facade --> Engines["XLS / XLSX / CSV engines"]
```

禁止形成 `easyexcel-web -> easyexcel-poem` 或 `easyexcel -> easyexcel-poem` 的反向依赖。

## 证据索引

| 声明 | 事实来源 |
|:---|:---|
| Extractor/请求行为 | [`src/excel_request.rs`](src/excel_request.rs) |
| Responder/响应行为 | [`src/excel_response.rs`](src/excel_response.rs) |
| 错误映射 | [`src/excel_error.rs`](src/excel_error.rs) |
| 可运行集成 | [`examples/poem`](https://github.com/easy-4-rust/easyexcel-rust/tree/main/examples/poem) |
| 共享适配器契约 | [`tests/easyexcel-web-conformance`](https://github.com/easy-4-rust/easyexcel-rust/tree/main/tests/easyexcel-web-conformance) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-poem)
- [easyexcel-web](https://crates.io/crates/easyexcel-web)
- [可运行示例](https://github.com/easy-4-rust/easyexcel-rust/tree/main/examples/poem)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [英文 README](README.md)
