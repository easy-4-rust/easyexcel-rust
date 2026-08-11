# Poem Excel Web Demo

Shows how to integrate `easyexcel-web` with the [Poem](https://github.com/poem-web/poem)
framework using Poem's `#[handler]` macro and `Data` extractor.

## Default port

**8083**.

## Quick start

```bash
cargo run -p easyexcel-demo-poem
```

The server listens on `http://127.0.0.1:8083` and provides:

| Method | Path | Description |
|---|---|---|
| `GET` | `/download` | Returns an XLSX file with 10 sample rows |
| `POST` | `/upload` | Accepts CSV, XLS, or XLSX for streaming parse |
| `GET` | `/health` | Returns plain-text "ok" (health check) |

## curl examples

### Download XLSX

```bash
curl -OJ http://127.0.0.1:8083/download
# Produces poem-example.xlsx
```

### Upload CSV

```bash
curl -X POST http://127.0.0.1:8083/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
```

### Upload XLSX

```bash
curl -X POST http://127.0.0.1:8083/upload \
  -H 'Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' \
  -H 'x-excel-file-name: data.xlsx' \
  --data-binary @data.xlsx
```

### Upload XLS (BIFF8)

```bash
curl -X POST http://127.0.0.1:8083/upload \
  -H 'Content-Type: application/vnd.ms-excel' \
  -H 'x-excel-file-name: legacy.xls' \
  --data-binary @legacy.xls
```

### Health check

```bash
curl http://127.0.0.1:8083/health
# ok
```

## Policy configuration

```rust
let runtime = ExcelWebRuntime::new(
    ExcelWebPolicy::new(ResourceLimits::default()).with_max_concurrent_tasks(4),
);
```

See the [Axum README](../axum/README.md#policy-configuration) for details.

## Framework-specific details

- Uses `poem::web::Data<&ExcelWebRuntime>` for dependency injection, injected
  via `AddData` middleware.
- `ExcelRequest<T>` implements Poem's `FromRequest` trait.
- `ExcelResponse<T>` implements Poem's `IntoResponse`.
- Error type is `ExcelPoemError`, which converts to `poem::Error`.
- Routes are registered with `Route::new().at("/download", get(handler))`.

## Conformance suite

```bash
cargo test -p easyexcel-web-conformance --test poem
```
