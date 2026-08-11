# Rocket Excel Web Demo

Shows how to integrate `easyexcel-web` with the [Rocket](https://rocket.rs/)
framework using Rocket's attribute macros and managed state.

## Default port

**8000**.

## Quick start

```bash
cargo run -p easyexcel-demo-rocket
```

The server listens on `http://127.0.0.1:8000` and provides:

| Method | Path | Description |
|---|---|---|
| `GET` | `/download` | Returns an XLSX file with 10 sample rows |
| `POST` | `/upload` | Accepts CSV, XLS, or XLSX for streaming parse |

## curl examples

### Download XLSX

```bash
curl -OJ http://127.0.0.1:8000/download
# Produces rocket-example.xlsx
```

### Upload CSV

```bash
curl -X POST http://127.0.0.1:8000/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
```

### Upload XLSX

```bash
curl -X POST http://127.0.0.1:8000/upload \
  -H 'Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' \
  -H 'x-excel-file-name: data.xlsx' \
  --data-binary @data.xlsx
```

### Upload XLS (BIFF8)

```bash
curl -X POST http://127.0.0.1:8000/upload \
  -H 'Content-Type: application/vnd.ms-excel' \
  -H 'x-excel-file-name: legacy.xls' \
  --data-binary @legacy.xls
```

## Policy configuration

```rust
let runtime = ExcelWebRuntime::new(
    ExcelWebPolicy::new(ResourceLimits::default()).with_max_concurrent_tasks(4),
);
```

See the [Axum README](../axum/README.md#policy-configuration) for details.

## Framework-specific details

- Uses Rocket's `#[get("/download")]` and `#[post("/upload", data = "<request>")]`
  attribute macros for route declaration.
- `ExcelWebRuntime` is managed via `rocket::build().manage(runtime)`.
- `ExcelRequest<T>` implements Rocket's `FromData` trait.
- `ExcelResponse<T>` implements Rocket's `Responder`.
- Error type is `ExcelRocketError`.
- The `#[launch]` attribute builds and returns the Rocket instance.

## Conformance suite

```bash
cargo test -p easyexcel-web-conformance --test rocket
```
