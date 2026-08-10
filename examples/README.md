# easyexcel-rust Examples

This directory contains ten runnable demos covering the three core capabilities
of `easyexcel-rust`: reading, writing, and template filling -- plus integration
examples for seven popular Rust web frameworks.

---

## CLI examples

These three examples run as standalone binaries and demonstrate the
library's synchronous read/write/fill API.

### read

Reads an XLSX or CSV file and prints each row to stdout.

```bash
cargo run -p easyexcel-demo-read -- path/to/data.xlsx
```

See [read/README.md](read/README.md) for details.

### write

Generates an XLSX file with sample data.

```bash
cargo run -p easyexcel-demo-write -- path/to/output.xlsx
```

See [write/README.md](write/README.md) for details.

### fill

Demonstrates template filling: creates a template with placeholders, then
produces a filled output file.

```bash
cargo run -p easyexcel-demo-fill
```

See [fill/README.md](fill/README.md) for details.

---

## Web framework examples

Seven web examples share the `easyexcel-web` unified runtime, with each
framework adapter crate exposing native `ExcelRequest<T>` and
`ExcelResponse<T>` types.

| Framework | Package | Default port | README |
|---|---|---:|---|
| Axum | `easyexcel-demo-axum` | 8080 | [axum/README.md](axum/README.md) |
| Actix Web | `easyexcel-demo-actix` | 8081 | [actix/README.md](actix/README.md) |
| Hyper | `easyexcel-demo-hyper` | 8082 | [hyper/README.md](hyper/README.md) |
| Poem | `easyexcel-demo-poem` | 8083 | [poem/README.md](poem/README.md) |
| Salvo | `easyexcel-demo-salvo` | 8084 | [salvo/README.md](salvo/README.md) |
| Warp | `easyexcel-demo-warp` | 8085 | [warp/README.md](warp/README.md) |
| Rocket | `easyexcel-demo-rocket` | 8000 | [rocket/README.md](rocket/README.md) |

Each example provides `GET /download` and `POST /upload`. Upload accepts raw
CSV, XLS, or XLSX request bodies:

```bash
# Start any framework (e.g. Axum)
cargo run -p easyexcel-demo-axum

# Upload CSV
curl -X POST http://127.0.0.1:8080/upload \
  -H 'Content-Type: text/csv' \
  -H 'x-excel-file-name: rows.csv' \
  --data-binary @rows.csv

# Download XLSX
curl -OJ http://127.0.0.1:8080/download
```

### Conformance test suite

The same upload/download behavior contract is verified across all seven
frameworks by the shared conformance suite at `tests/easyexcel-web-conformance/`.
This ensures the `easyexcel-web` runtime produces identical results regardless
of the HTTP adapter, preventing divergence between examples and production
adapters in resource limits, error protocol, or streaming semantics.

Run all conformance tests:

```bash
cargo test -p easyexcel-web-conformance
```

Run a single framework's conformance test:

```bash
cargo test -p easyexcel-web-conformance --test axum
```
