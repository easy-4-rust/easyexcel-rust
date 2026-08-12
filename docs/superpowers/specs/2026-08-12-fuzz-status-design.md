# Fuzz Testing Status

**Date:** 2026-08-11
**cargo-fuzz version:** 0.13.2
**Nightly toolchain:** rustc 1.99.0-nightly (2026-08-10)

## Overview

easyexcel-rust has 5 fuzz targets covering all major parsing entry points:
XLSX (OOXML), XLS (BIFF8), CSV, formula, and Markdown GFM tables.

## Targets

| Target | Crate Under Test | API Fuzzed | What It Covers |
|--------|-----------------|------------|----------------|
| `fuzz_xlsx_parse` | `easyexcel-xlsx` | `read(Cursor<&[u8]>)` | ZIP extraction, OOXML XML parsing, shared strings, styles, cell parsing |
| `fuzz_xls_parse` | `easyexcel-xls` | `read(Cursor<&[u8]>)` | CFB/OLE2 container, BIFF8 record parsing, SST, encryption detection |
| `fuzz_csv_parse` | `easyexcel-csv` | `read_csv(Cursor<&[u8]>, &CsvReadOptions)` | Delimiter auto-detection, BOM handling, encoding transcoding, type inference |
| `fuzz_formula_parse` | `easyexcel-formula` | `parse_detailed(&str)` | Formula lexer, Pratt parser, AST construction |
| `fuzz_markdown_parse` | `easyexcel-markdown` | `read_markdown(Cursor<&[u8]>, &MarkdownImportOptions)` | GFM table parsing, UTF-8 validation, type inference |

## Initial Run Results (100 iterations each, seeded corpus)

| Target | Coverage Edges | Features | Corpus Size | Panics |
|--------|---------------|----------|-------------|--------|
| `fuzz_xlsx_parse` | 2237 | 3101 | 24 entries / 154KB | 0 |
| `fuzz_xls_parse` | 1533 | 2082 | 21 entries / 365KB | 0 |
| `fuzz_csv_parse` | 749 | 1087 | 36 entries / 99B | 0 |
| `fuzz_formula_parse` | 382 | 783 | 44 entries / 202B | 0 |
| `fuzz_markdown_parse` | 574 | 727 | 30 entries / 64B | 0 |

**No panics detected in initial 100-iteration runs.**

## How to Run

### Prerequisites

- Nightly Rust toolchain: `rustup install nightly`
- cargo-fuzz: `cargo install cargo-fuzz`

### Run a Single Target

```bash
cd easyexcel-rust
cargo +nightly fuzz run fuzz_xlsx_parse
```

### Run with Limited Iterations

```bash
cargo +nightly fuzz run fuzz_formula_parse -- -runs=10000
```

### Run with Time Limit

```bash
cargo +nightly fuzz run fuzz_csv_parse -- -max_total_time=3600
```

### Run All Targets

```bash
for target in fuzz_xlsx_parse fuzz_xls_parse fuzz_csv_parse fuzz_formula_parse fuzz_markdown_parse; do
    cargo +nightly fuzz run "$target" -- -max_total_time=600
done
```

## Corpus

Each target has a seeded corpus in `fuzz/corpus/<target>/`:

- **XLSX/XLS:** Real conformance test files from `tests/easyexcel-web-conformance/src/fixtures/`
- **CSV:** Real migration file-map from `docs/data/migration/file-map.csv`
- **Formula:** Hand-crafted Excel formulas (`=SUM(A1:A10)`, `=IF(...)`)
- **Markdown:** GFM table with headers and data rows

The fuzzer mutates these seeds to explore new code paths.

## Architecture Notes

- All targets use `#![no_main]` + `libfuzzer-sys` (standard cargo-fuzz pattern).
- Each target calls the real public API -- no mocks, no shortcuts.
- Panics are NOT caught; a panic = a real bug to report (not suppress).
- The `fuzz/` directory is excluded from the workspace (`workspace.exclude`).
- Requires nightly toolchain (libfuzzer needs `-Zsanitizer=address`).

## Next Steps

1. **Longer runs:** Run each target for 1+ hours to explore deeper code paths.
2. **Seed more corpus entries:** Add edge-case files (empty, huge, encrypted, password-protected).
3. **CI integration:** Add a nightly CI job that runs each target for ~10 minutes.
4. **Coverage-guided dictionary:** Use `cargo fuzz cmin` to minimize corpus.
5. **Address sanitizer (ASan):** Already enabled by default in cargo-fuzz.
