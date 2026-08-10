# Actix Web Excel Demo

Shows how to integrate `easyexcel-web` with the [Actix Web](https://actix.rs/)
framework using native `ExcelRequest<T>` / `ExcelResponse<T>` types.

## Default port

**8081** (override with `PORT` environment variable).

## Quick start

```bash
cargo run -p easyexcel-demo-actix
```

The server listens on `http://127.0.0.1:8081` and provides two endpoints:

| Method | Path | Description |
|---|---|---|
| `GET` | `/download` | Returns an XLSX file with 10 sample rows |
| `POST` | `/upload` | Accepts CSV, XLS, or XLSX for streaming parse |

## curl examples

### Download XLSX

```bash
curl -OJ http://127.0.0.1:8081/download
```

### Upload CSV

```bash
curl -X POST http://127.0.0.1:8081/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
```

### Upload XLSX

```bash
curl -X POST http://127.0.0.1:8081/upload \
  -H 'Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' \
  -H 'x-excel-file-name: data.xlsx' \
  --data-binary @data.xlsx
```

### Upload XLS (BIFF8)

```bash
curl -X POST http://127.0.0.1:8081/upload \
  -H 'Content-Type: application/vnd.ms-excel' \
  -H 'x-excel-file-name: legacy.xls' \
  --data-binary @legacy.xls
```

## Policy configuration

```rust
let policy = ExcelWebPolicy::new(ResourceLimits::default())
    .with_max_concurrent_tasks(4)
    .with_row_channel_capacity(32);
```

See the [Axum README](../axum/README.md#policy-configuration) for a detailed
explanation of each parameter; the policy API is identical across all seven
framework examples.

## Framework-specific details

- Uses `web::Data<ExcelWebRuntime>` for dependency injection (Actix's managed
  state pattern, equivalent to Axum's `State`).
- `ExcelRequest<T>` implements Actix's `FromRequest` trait.
- `ExcelResponse<T>` implements Actix's `HttpResponse` conversion.
- Error type is `ExcelActixError`, which maps to appropriate HTTP status codes
  and problem-detail JSON bodies.
- The `#[actix_web::main]` macro sets up the Actix runtime (equivalent to
  Axum's `#[tokio::main]`).

## Conformance suite

This example is covered by the shared conformance test suite at
`tests/easyexcel-web-conformance/`.

```bash
cargo test -p easyexcel-web-conformance --test actix
```
