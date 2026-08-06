# easyexcel-web

[简体中文](README.zh-CN.md)

Framework-neutral Web import/export runtime for EasyExcel-Rust.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Provides bounded uploads, temporary-file lifecycle, typed row streams and streaming downloads.
- Centralizes resource limits, backpressure, cancellation, timeout, tracing and stable problem details.

## Architecture

```text
HTTP body -> ExcelImport -> bounded rows -> application -> ExcelExport -> HTTP body
```

Main public surface: `ExcelImport, ExcelRows, ExcelExport, ExcelWebPolicy, ExcelWebRuntime, ExcelProblemDetails`.

## Installation and usage

```toml
[dependencies]
easyexcel-web = "0.1.1"
```

```rust
use easyexcel_web::{ExcelExport, ExcelImport, ExcelWebPolicy, ExcelWebRuntime};
```

## Compatibility and limits

This crate does not expose a framework extractor or responder. Select one of the dedicated framework adapters.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Streaming and framework contract

All seven adapters expose `ExcelRequest<T>` and `ExcelResponse<T>` while retaining their framework-native extraction and response mechanisms. Upload metadata is resolved from `x-excel-file-name`, `Content-Disposition` or `Content-Type`; `x-request-id` is propagated into tracing and stable error responses.

XLSX and legacy XLS parsers require random access to a complete container. Therefore streaming upload means chunked request-body spooling to an automatically cleaned temporary file, followed by bounded row delivery; it does not mean buffering the entire file in a `Vec<u8>`. Downloads are generated before response streaming begins so failures do not emit a partially valid spreadsheet.

V1 enforces file-byte and total-row limits. Worksheet-count and formula-cell limits remain dependent on uniform counting hooks in the format engines and are not claimed as enforced until those hooks are connected. Runnable adapters live under `examples/{axum,actix,hyper,poem,rocket,salvo,warp}` and share `tests/easyexcel-web-conformance`.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-web)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
