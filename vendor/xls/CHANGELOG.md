# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Streaming reads (huge files)
- `export --stream` streams a sheet row-by-row to `csv`/`tsv`/`jsonl` without
  loading the whole workbook into memory — for `.xlsx`/`.csv` inputs too large to
  fit comfortably in RAM. Output is identical to the in-memory path (a 100k-row
  export used ~3.5× less peak memory in testing). Uses cached formula values (no
  recalculation); other formats/inputs transparently fall back to the in-memory
  exporter. Backed by a new `core::stream` API (`RowSink`/`stream_path`).

#### Named ranges & Excel tables
- **Excel table objects** (`ListObject`s) now read + write with a lossless
  round-trip: the `<table>` part, worksheet `<tableParts>`, sheet relationships,
  and content-type overrides are all modeled and regenerated. Original table XML
  is preserved verbatim so styles/autofilter/calculated columns survive.
- **Structured references** in formulas: `Sales[Amount]`, `Sales[#All]`,
  `Sales[#Headers]`, `Sales[[#Data],[Amount]]`, and a bare `Sales` (→ data body).
  (The `[@…]` this-row form is not supported.)
- `table add|list|rm` — create a table over a range (headers from the first row,
  or `--no-header`), list tables, or remove one.
- `name add|list|rm` — manage defined names (named ranges) from the CLI.
- `get` resolves a defined name, a table name, or a structured reference in place
  of an A1 range; `info` lists named ranges and tables.

#### Headless reads (agent pipelines)
- `get` accepts a **range** (e.g. `A1:J200`), not just a single cell, with
  `--format table|csv|tsv|json|jsonl|md`. A bare single cell with default
  options still prints just its value (back-compatible).
- `--raw` on `get`/`eval`/`export` emits stored values (no thousands
  separators, booleans as `true`/`false`); `--dates iso|serial` controls how
  date-formatted cells render. JSON output is always typed.
- `--format json --header` returns an array of objects keyed by the header row;
  `jsonl` emits one record per line.
- Array/range-returning `eval` (e.g. `=C2:C9`) renders as a grid in the chosen
  format instead of collapsing to a scalar.
- `export -o -` streams text formats (csv/tsv/json/jsonl/md) to stdout; binary
  formats (xlsx/xls) refuse stdout with a clear error. New `tsv`/`json`/`jsonl`/
  `md` export formats.
- `format` — print a cell's number format (`DATE dd/mm/yyyy`, `NUMBER 0.00`, …).

#### Data manipulation & discovery verbs
- `query` — read-only **SQL `SELECT`** over sheets-as-tables: `WHERE`
  (`= != <> < <= > >= LIKE`, `AND/OR/NOT`, parens), `GROUP BY` with
  `SUM/COUNT/AVG/MIN/MAX`, `ORDER BY` (name/alias/ordinal, ASC/DESC), `LIMIT`,
  and an equi-`JOIN` across sheets. Columns by header name or letter.
- `pivot` — group by a column and aggregate another (`sum|count|mean|min|max`).
- `filter` — print rows matching a predicate (`amount>1000`, `cat==fuel`,
  `col:number`/`col:text`).
- `join` — inner-join two sheets/files on a key column.
- `diff --key <col>` — keyed, row-wise diff (added/removed/changed by key),
  alongside the existing positional cell-by-cell diff.
- `sort` (stable, multi-key), `dedup` (by key column(s)), `append` (align by
  header name).
- `profile` — column stats plus a warning when numbers **or dates** are stored
  as text.
- `grep` (matches with cell addresses), `head`, `tail`.

#### Write-side
- `to-date` — parse text-stored dates (e.g. `"04/04/2025"`) into real date
  serials and apply the format, the date twin of `to-number`. Excel-style
  format tokens, day-first safe (`m` is minute next to `h`/`s`, else month).
- `format-set` (number format on a range), `autofit` (column widths), `style`
  (bold/italic/font color/fill).
- `batch` — apply many `--set CELL=VALUE` edits in one atomic open/recalc/save.

#### Safety
- Global `--dry-run` (print the diff, write nothing), `--backup` (write
  `<file>.bak`), and `--output <PATH>` (write a copy) on every mutating command.
- Write commands accept `-` (CSV on stdin) when paired with `--output`.

#### Dynamic arrays & LAMBDA (formula engine)
- **Array-aware operators**: binary/unary ops over an array or multi-cell range
  broadcast element-wise (`A1:A10>5` → a boolean array; `range*2` → an array).
- **Spill engine**: a formula returning an array (or a bare multi-cell range)
  spills into neighboring cells; `#SPILL!` on obstruction. Derived state, rebuilt
  on recalc; the CLI recalculates on open so spills appear across invocations.
- **`[DA]` functions**: `SORT`, `SORTBY`, `UNIQUE`, `FILTER`, `SEQUENCE`,
  `RANDARRAY`, `VSTACK`, `HSTACK`, `TOROW`, `TOCOL`, `WRAPROWS`, `WRAPCOLS`,
  `TAKE`, `DROP`, `EXPAND`, `CHOOSEROWS`, `CHOOSECOLS`, `TRIMRANGE`, `TEXTSPLIT`,
  `MODE.MULT`.
- **`[LAMBDA]` functions**: first-class `LAMBDA` values, `LET`, `MAP`, `REDUCE`,
  `SCAN`, `BYROW`, `BYCOL`, `MAKEARRAY`, `ISOMITTED`. With these, **every
  standard Excel worksheet function is implemented.**
- Bare single-cell-reference formulas (`=A1`) now deref to the cell value.

#### Other formula functions
- `PERCENTOF`, and the legacy compatibility aliases (`MODE`, `COVAR`,
  `NORMDIST`/`NORMINV`/`NORMSDIST`/`NORMSINV`, `LOGNORMDIST`/`LOGINV`,
  `BETADIST`/`BETAINV`, `GAMMADIST`/`GAMMAINV`, `BINOMDIST`, `NEGBINOMDIST`,
  `HYPGEOMDIST`, `POISSON`, `EXPONDIST`, `WEIBULL`, `CHIDIST`/`CHIINV`,
  `FDIST`/`FINV`, `TDIST`/`TINV`, `ZTEST`, `CONFIDENCE`, `CRITBINOM`).

## [0.1.0] — 2026-06-06

Initial baseline (unpublished):

- Read/write **XLSX** (OOXML) and **XLS** (BIFF8), read/write **CSV**, with
  opaque round-trip of unknown parts/records.
- Built-in **formula engine** (lexer/parser/evaluator + dependency-ordered
  recalculation) with 450+ worksheet functions.
- Decryption of password-protected `.xlsx` (ECMA-376 agile + standard).
- CLI: `info`/`get`/`set`/`eval`/`export`/`import`/`diff`, range edits
  (`clear`/`fill`/`copy`/`move`), row/column insert/delete, sheet management,
  and `to-number`.
- Interactive **TUI** (ratatui): navigation, in-cell + top-bar editing, mouse,
  scrollbars, column-width drag, range highlighting.
