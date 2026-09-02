# easyexcel-salvo

[简体中文](README.zh-CN.md)

Native EasyExcel request extraction and response adapter for Salvo.

> Release: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0
>
> Last updated: 2026-08-11 · Status: active

## Overview

`easyexcel-salvo` only bridges Salvo transport types to `easyexcel-web`. Upload spooling, resource limits, row-stream backpressure, cancellation, timeouts, temporary-file cleanup and stable errors remain in the shared runtime, preventing framework-specific semantic drift.

Native integration: `Extractible` request type and Salvo `Writer` response. Runtime injection: `ExcelWebRuntime` inserted into request extensions by a hoop.

## At a glance

```text
HTTP request -> easyexcel-salvo -> easyexcel-web -> typed rows / streamed response
```

## Architecture

```mermaid
flowchart LR
    Request["Salvo request"] --> Adapter["easyexcel-salvo"]
    Adapter --> Import["easyexcel-web / ExcelImport"]
    Import --> Rows["ExcelRows<T> / backpressure"]
    Rows --> Handler["Application handler"]
    Handler --> Export["ExcelExport<T>"]
    Export --> Response["Salvo response"]
```

The adapter does not reimplement spreadsheet parsing, writing or resource policy. Business rows are consumed through a bounded channel and downloads are exposed to Salvo as an asynchronous file stream.

## Capabilities and Boundaries

| What easyexcel-salvo does | What easyexcel-salvo does NOT do |
|:---|:---|
| `Extractible` request type for typed backpressured row stream | Upload spooling / resource limits / timeouts (in `easyexcel-web`) |
| Salvo `Writer` response for streaming XLSX/XLS/CSV download | Business validation, authorization or persistence |
| `ExcelSalvoError` mapping to Salvo error protocol | Reimplementing spreadsheet parsing or writing |
| Hoop-based `ExcelWebRuntime` injection into request extensions | TUI / HTML form handling |

## Capability matrix

| Capability | Status | Implementation |
|:---|:---|:---|
| `ExcelRequest<T>` | Available | Native Salvo extraction with a typed, backpressured row stream. |
| `ExcelResponse<T>` | Available | Generates a controlled file before committing headers, then streams it asynchronously. |
| Limits and concurrency | Shared | `ExcelWebPolicy` + `ExcelWebRuntime` |
| Error protocol | Stable | `ExcelSalvoError` + `ExcelProblemDetails` |
| TUI / HTML form | Out of scope | Owned by the application or examples. |

## Installation

```toml
[dependencies]
easyexcel = "0.1.3"
easyexcel-salvo = "0.1.3"
```

All workbook APIs remain under `easyexcel::...`; only Salvo-native extractor, writer and error types come from this adapter. The adapter depends on `easyexcel`, so facade-side re-export would create a cycle. Keep both crates on the same release line.

## Usage from examples

The runnable example is in [`examples/salvo`](https://github.com/easy-4-rust/easyexcel-rust/tree/main/examples/salvo). Default port: **8084**.

```bash
cargo run -p example-salvo
# Listening on http://127.0.0.1:8084
# POST /upload   - upload an Excel file
# GET  /download - download a sample XLSX
```

## Define the row model

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

## Streaming download

```rust
use easyexcel::io::Format;
use easyexcel_salvo::{ExcelResponse, ExcelWebRuntime};
use salvo::prelude::*;

#[handler]
async fn download(
    request: &mut Request,
    depot: &mut Depot,
    response: &mut Response,
) {
    let runtime = request.extensions()
        .get::<ExcelWebRuntime>()
        .expect("runtime attached")
        .clone();
    match ExcelResponse::<ReportRow>::prepare(
        report_rows(),
        Format::Xlsx,
        "report.xlsx",
        "Data",
        runtime.generated_context(),
    ).await {
        Ok(value) => value.write(request, depot, response).await,
        Err(error) => error.write(request, depot, response).await,
    }
}
```

`ExcelResponse::prepare` completes generation and limit checks before returning a Salvo response. The response body then reads the temporary file asynchronously instead of copying the complete file into memory.

## Backpressured upload

```rust
use easyexcel_salvo::{ExcelRequest, ExcelSalvoError};
use salvo::prelude::*;

#[handler]
async fn upload(
    request: &mut Request,
    depot: &mut Depot,
    response: &mut Response,
) {
    match ExcelRequest::<ReportRow>::extract(request, depot).await {
        Ok(value) => {
            let request_id = value.request_id().to_owned();
            let mut rows = value.into_rows();
            while let Some(row) = rows.next_row().await {
                if let Err(error) = row {
                    ExcelSalvoError::new(error, &request_id)
                        .write(request, depot, response).await;
                    return;
                }
            }
            response.render("success");
        }
        Err(error) => error.write(request, depot, response).await,
    }
}
```

Uploads must provide `x-excel-file-name`, `Content-Disposition` or a recognizable `Content-Type`. Optional `x-request-id` is propagated into tracing and error responses.

## Runtime wiring

```rust
use easyexcel_salvo::{ExcelWebPolicy, ExcelWebRuntime};
use salvo::prelude::*;

let runtime = ExcelWebRuntime::new(ExcelWebPolicy::default());
// Add a Salvo hoop that inserts runtime.clone() into request.extensions_mut().
// Then register /download and /upload handlers on Router.
```

Create one shared `ExcelWebRuntime` instead of rebuilding the concurrency permit pool per request. `ExcelWebPolicy` configures file bytes, rows, upload/processing timeouts, maximum tasks, row-channel capacity and temporary directory.

## Headers and errors

- `Content-Type` is derived from XLSX, XLS or CSV format.
- `Content-Disposition` uses UTF-8 filename encoding and sanitizes unsafe names.
- `Content-Length` comes from the generated file size.
- `ExcelSalvoError` maps shared failures to the framework-native rejection/error/response.
- Diagnostics go to tracing; the stable problem response does not expose internal paths.

## Capability boundaries

- Streaming upload means chunked spooling followed by parsing; it does not make XLS/XLSX random-access containers parseable before the complete upload arrives.
- Streaming download starts after successful generation so clients do not receive a partially valid workbook.
- The adapter does not own business validation, authorization or persistence; those belong to application handlers/middleware.
- The complete runnable service is in `examples/salvo`; shared assertions live in `tests/easyexcel-web-conformance`.

## Dependency relationship

```mermaid
flowchart TB
    Framework["Salvo"] --> Adapter["easyexcel-salvo"]
    Adapter --> Web["easyexcel-web"]
    Web --> Facade["easyexcel"]
    Facade --> Engines["XLS / XLSX / CSV engines"]
```

Reverse dependencies such as `easyexcel-web -> easyexcel-salvo` or `easyexcel -> easyexcel-salvo` are forbidden.

## Evidence map

| Claim | Source of truth |
|:---|:---|
| Extractor/request behavior | [`src/excel_request.rs`](src/excel_request.rs) |
| Responder/reply behavior | [`src/excel_response.rs`](src/excel_response.rs) |
| Error mapping | [`src/excel_error.rs`](src/excel_error.rs) |
| Runnable integration | [`examples/salvo`](https://github.com/easy-4-rust/easyexcel-rust/tree/main/examples/salvo) |
| Shared adapter contract | [`tests/easyexcel-web-conformance`](https://github.com/easy-4-rust/easyexcel-rust/tree/main/tests/easyexcel-web-conformance) |

## Related links

- [Repository](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-salvo)
- [easyexcel-web](https://crates.io/crates/easyexcel-web) -- shared Web execution kernel
- [Web conformance suite](https://github.com/easy-4-rust/easyexcel-rust/tree/main/tests/easyexcel-web-conformance)
- [Runnable example](https://github.com/easy-4-rust/easyexcel-rust/tree/main/examples/salvo)
- [Compatibility matrix](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [Chinese README](README.zh-CN.md)
