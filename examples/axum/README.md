# Axum Excel Web Demo

Shows how to integrate `easyexcel-web` with the [Axum](https://github.com/tokio-rs/axum)
framework using native `ExcelRequest<T>` / `ExcelResponse<T>` extractors.

## Default port

**8080** (override with `PORT` environment variable).

## Quick start

```bash
cargo run -p easyexcel-demo-axum
```

The server listens on `http://127.0.0.1:8080` and provides two endpoints:

| Method | Path | Description |
|---|---|---|
| `GET` | `/download` | Returns an XLSX file with 10 sample rows |
| `POST` | `/upload` | Accepts CSV, XLS, or XLSX for streaming parse |

## curl examples

### Download XLSX

```bash
curl -OJ http://127.0.0.1:8080/download
# Produces 测试.xlsx
```

### Upload CSV

```bash
curl -X POST http://127.0.0.1:8080/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
```

### Upload XLSX

```bash
curl -X POST http://127.0.0.1:8080/upload \
  -H 'Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' \
  -H 'x-excel-file-name: data.xlsx' \
  --data-binary @data.xlsx
```

### Upload XLS (BIFF8)

```bash
curl -X POST http://127.0.0.1:8080/upload \
  -H 'Content-Type: application/vnd.ms-excel' \
  -H 'x-excel-file-name: legacy.xls' \
  --data-binary @legacy.xls
```

## Policy configuration

The demo configures `ExcelWebPolicy` with resource limits:

```rust
let policy = ExcelWebPolicy::new(ResourceLimits::default())
    .with_max_concurrent_tasks(4)       // max concurrent uploads
    .with_row_channel_capacity(32);     // backpressure channel size
```

- `ResourceLimits::default()` sets sensible defaults for max file size, max
  rows, and max columns. Override individual fields as needed.
- `with_max_concurrent_tasks` controls how many uploads can be processed in
  parallel.
- `with_row_channel_capacity` sets the mpsc channel buffer between the parser
  worker and the consumer; lower values increase backpressure.

## Framework-specific details

- Uses `axum::extract::State<ExcelWebRuntime>` for dependency injection.
- `ExcelRequest<T>` is an Axum extractor (implements `FromRequest`).
- `ExcelResponse<T>` implements `IntoResponse`.
- Graceful shutdown via `SIGTERM`/`SIGINT` signal handling with
  `axum::serve(...).with_graceful_shutdown(...)`.

## Conformance suite

This example is one of seven frameworks covered by the shared conformance test
suite at `tests/easyexcel-web-conformance/`. The conformance tests verify
identical upload/download behavior across all frameworks, ensuring the
`easyexcel-web` runtime produces consistent results regardless of the HTTP
adapter.

Run the Axum conformance test:

```bash
cargo test -p easyexcel-web-conformance --test axum
```
