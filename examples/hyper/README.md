# Hyper Excel Web Demo

Shows how to integrate `easyexcel-web` with [Hyper](https://hyper.rs/) using
the low-level `service_fn` API for maximum control over HTTP handling.

## Default port

**8082**.

## Quick start

```bash
cargo run -p easyexcel-demo-hyper
```

The server listens on `http://127.0.0.1:8082` and provides two endpoints:

| Method | Path | Description |
|---|---|---|
| `GET` | `/download` | Returns an XLSX file with 10 sample rows |
| `POST` | `/upload` | Accepts CSV, XLS, or XLSX for streaming parse |

Any other method/path returns a `404 Not Found` plain-text response.

## curl examples

### Download XLSX

```bash
curl -OJ http://127.0.0.1:8082/download
# Produces hyper-example.xlsx
```

### Upload CSV

```bash
curl -X POST http://127.0.0.1:8082/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv
```

### Upload XLSX

```bash
curl -X POST http://127.0.0.1:8082/upload \
  -H 'Content-Type: application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' \
  -H 'x-excel-file-name: data.xlsx' \
  --data-binary @data.xlsx
```

### Upload XLS (BIFF8)

```bash
curl -X POST http://127.0.0.1:8082/upload \
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

See the [Axum README](../axum/README.md#policy-configuration) for details on
`ResourceLimits` and concurrency tuning.

## Framework-specific details

- Uses raw Hyper `service_fn` + `http1::Builder` instead of a routing
  framework. Method and path matching is done manually via
  `request.method().as_str()` and `request.uri().path()`.
- `ExcelRequest<T>::from_request(request, &runtime)` parses the incoming
  Hyper `Request<Incoming>` directly.
- `ExcelResponse<T>::into_response()` converts to a Hyper
  `Response<ResponseBody>`.
- Error handling via `ExcelHyperError::into_response()`, which returns
  structured problem-detail JSON.
- The server loops on `listener.accept()` and spawns a task per connection,
  using `hyper_util::rt::TokioIo` to bridge Hyper's IO traits with Tokio.

## Conformance suite

```bash
cargo test -p easyexcel-web-conformance --test hyper
```
