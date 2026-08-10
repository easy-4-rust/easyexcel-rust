# Salvo Excel Web Demo

Shows how to integrate `easyexcel-web` with the [Salvo](https://salvo.rs/)
framework using Salvo's handler macro and `Extractible` trait.

## Minimum supported Rust version (MSRV)

This example requires **Rust 1.89** or later due to Salvo's MSRV.

## Default port

**8084**.

## Quick start

```bash
cargo run -p easyexcel-demo-salvo
```

The server listens on `http://127.0.0.1:8084` and provides:

| Method | Path | Description |
|---|---|---|
| `GET` | `/download` | Returns an XLSX file with 10 sample rows |
| `POST` | `/upload` | Accepts CSV, XLS, or XLSX for streaming parse |

## curl examples

### Download XLSX

```bash
curl -OJ http://127.0.0.1:8084/download
# Produces salvo-example.xlsx
```

### Upload CSV

```bash
curl -X POST http://127.0.0.1:8084/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
```

### Upload XLSX

```bash
curl -X POST http://127.0.0.1:8084/upload \
  -H 'Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' \
  -H 'x-excel-file-name: data.xlsx' \
  --data-binary @data.xlsx
```

### Upload XLS (BIFF8)

```bash
curl -X POST http://127.0.0.1:8084/upload \
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

- Uses a custom `RuntimeHoop` (Salvo's middleware/hoop pattern) to inject
  `ExcelWebRuntime` into each request's extensions.
- `ExcelRequest<T>` is extracted via `ExcelRequest::<T>::extract(request, depot)`
  (Salvo's `Extractible` trait).
- `ExcelResponse<T>` writes directly via `.write(request, depot, response)`.
- Error type is `ExcelSalvoError`, which also writes directly to the response.
- Routes are registered with `Router::new().push(Router::with_path("download").get(handler))`.

## Conformance suite

```bash
cargo test -p easyexcel-web-conformance --test salvo
```
