# Warp Excel Web Demo

Shows how to integrate `easyexcel-web` with the [Warp](https://github.com/seanmonstar/warp)
framework using Warp's `Filter` combinators.

## Default port

**8085**.

## Quick start

```bash
cargo run -p easyexcel-demo-warp
```

The server listens on `http://127.0.0.1:8085` and provides:

| Method | Path | Description |
|---|---|---|
| `GET` | `/download` | Returns an XLSX file with 10 sample rows |
| `POST` | `/upload` | Accepts CSV, XLS, or XLSX for streaming parse |

## curl examples

### Download XLSX

```bash
curl -OJ http://127.0.0.1:8085/download
# Produces warp-example.xlsx
```

### Upload CSV

```bash
curl -X POST http://127.0.0.1:8085/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
```

### Upload XLSX

```bash
curl -X POST http://127.0.0.1:8085/upload \
  -H 'Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' \
  -H 'x-excel-file-name: data.xlsx' \
  --data-binary @data.xlsx
```

### Upload XLS (BIFF8)

```bash
curl -X POST http://127.0.0.1:8085/upload \
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

- Uses Warp's `Filter` combinators: `warp::path("download").and(warp::get())`
  and `warp::path("upload").and(warp::post()).and(excel_request::<T>(runtime))`.
- The `excel_request::<T>(runtime)` filter (from `easyexcel_warp`) extracts
  the request body into an `ExcelRequest<T>`.
- `ExcelResponse<T>` implements Warp's `Reply` trait.
- Error handling uses Warp's rejection model: `ExcelWarpRejection` is a custom
  rejection, recovered via `recover_excel_rejection`.
- Routes are combined with `.or()` and served via `warp::serve(...).run(...)`.

## Conformance suite

```bash
cargo test -p easyexcel-web-conformance --test warp
```
