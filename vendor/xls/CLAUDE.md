# xls — project conventions

Terminal spreadsheet (XLS/XLSX/CSV read+write, formula engine, CLI+TUI). Single
publishable crate `xls`; `core`/`cli`/`tui` are internal modules (one publishable
artifact — features keep `core` free of UI deps).

## Layout
- `src/core/` — data model, formula engine, file I/O. Always compiled, no UI deps.
  - `model.rs` Workbook/Sheet/Cell · `addr.rs` A1/R1C1 · `dates.rs` · `numfmt.rs`
  - `styles.rs` · `value.rs` CellValue · `csv.rs` · `stream.rs` (memory-bounded
    row reader) · `xlsx/` (incl. `tables.rs` table objects) · `xls/`
  - `formula/` — `ast` `parse` `value`(Value) `coerce` `context`(Context trait)
    `eval`(Evaluator) `engine`(Engine recalc) `functions/`(registry + categories)
- `src/cli/` — clap front-end (feature `cli`).
- `src/tui/` — ratatui UI (feature `tui`).

## Build / test
- `cargo test --no-default-features --lib` — fast core-only check.
- `cargo build` / `cargo test` — full (pulls clap/ratatui).
- `cargo clippy --all-targets` · `cargo fmt`.

## Conventions
- Functions: implement in `formula/functions/<category>.rs`, register in that
  file's `register(&mut Registry)`, case-insensitive name, arity + Excel error
  semantics. KAT tests using `functions::testutil::TestCtx`. Lazy special forms
  (IF/AND/OR/IFERROR/IFNA/CHOOSE/IFS/SWITCH) live in `formula/eval.rs`, not the
  registry. Record intentional Excel deviations as `// PARITY:` comments.
- Opaque round-trip: preserve unknown parts/records as `OpaquePart` bytes.
- Format code before committing.
