# xls

**A spreadsheet for your terminal.** `xls` reads and writes Excel `.xls`
(BIFF8), `.xlsx` (OOXML) and `.csv` files, evaluates Excel formulas with a
built-in engine covering all 522 worksheet functions (incl. dynamic-array spill
and `LAMBDA`), and ships both a scriptable
CLI and an interactive, mouse-aware TUI — all in a single Rust crate with no
external runtime.

```
┌ Data ───────────────────────────────────────────────┐
│     A          B          C          D               │
│ 1  Region    Units      Price      Total             │
│ 2  North       120      12.50    =B2*C2  → 1500.00   │
│ 3  South        90      12.50    =B3*C3  → 1125.00   │
│ 4  ────────────────────────────────────────          │
│ 5  Total    =SUM(B2:B3)        =SUM(D2:D3) → 2625.00 │
└──────────────────────────────────────────────────────┘
 NORMAL  D5  =SUM(D2:D3)  number                    *modified
```

> Status: pre-release (`0.1`). The core (data model, formula engine, file I/O)
> is well-tested, including dynamic-array spill and `LAMBDA`.

## Why

- **It writes `.xls`.** No other pure-Rust crate writes the legacy BIFF8 binary
  format. `xls` owns the format end-to-end (read *and* write).
- **Real formulas.** A from-scratch engine — lexer, Pratt parser, dependency
  graph, topological recalculation, circular-reference detection — not a
  thin wrapper.
- **Round-trip fidelity.** Styles, number formats, merged cells and frozen panes
  are preserved; unrecognized parts (charts, drawings, pivots, VBA) are kept
  verbatim so saving never destroys data.
- **One crate, one binary.** `core` / `cli` / `tui` are internal modules behind
  Cargo features, so `cargo install xls` ships exactly one artifact and the
  library build never pulls UI dependencies.

## Install

```sh
# from crates.io (once published)
cargo install xls

# from source
git clone https://github.com/zemse/xls && cd xls
cargo install --path .
```

## CLI

```sh
# Read / inspect
xls info report.xlsx                 # format, sheets, dimensions, date system
xls get report.xlsx Sheet1!B2        # print a cell's displayed value
xls get report.xlsx 'A1:J200' --format csv   # dump a range (table|csv|tsv|json|jsonl|md)
xls get report.xlsx 'A1:J200' --format json --header  # array of objects keyed by header row
xls get report.xlsx 'A1:J200' --raw --dates iso       # stored values; dates as ISO/serial
xls eval data.csv '=AVERAGE(A1:A10)' # evaluate a formula against a file
xls eval data.csv '=C2:C9' --format csv               # array results print as a grid
xls format report.xlsx C2            # print a cell's number format (DATE/NUMBER/GENERAL)
xls head report.xlsx -n 20           # first / last N rows (head / tail)
xls grep report.xlsx ZANMAI          # matching cells with their addresses
xls profile report.xlsx amount       # column stats + "numbers stored as text" warning
xls diff before.xlsx after.xlsx --key date  # keyed row diff (omit --key for cell-by-cell)

# Reshape / combine (read-only verbs print to stdout)
xls pivot report.xlsx --rows category --values amount --agg sum   # group + aggregate
xls filter report.xlsx 'amount>1000' --format csv                 # rows matching a predicate
xls join a.xlsx b.xlsx --on id       # inner-join two sheets on a key
xls query report.xlsx "SELECT category, SUM(amount) AS total FROM Sheet1 GROUP BY category ORDER BY total DESC"
                                     # read-only SQL: sheets are tables (row 0 = header), WHERE/GROUP BY/JOIN/ORDER BY/LIMIT

# Write / mutate (each edits in place and saves)
xls new book.xlsx                    # create an empty workbook
xls set report.xlsx A1 '=SUM(B:B)'   # set a cell (formula/number/text), recalc, save
xls batch report.xlsx --set A1=1 --set B2=hi   # many edits, one atomic open/save
xls clear report.xlsx A1:B10         # clear a range
xls fill report.xlsx A1:A10 0        # set a whole range to one value
xls sort report.xlsx --by amount --desc        # stable multi-key sort (header kept)
xls dedup report.xlsx --on id        # drop duplicate rows by key column(s)
xls append base.xlsx new.xlsx        # append rows, aligned by header name
xls to-number report.xlsx H1:H200    # force-convert any numeric text in a range
xls to-date report.xlsx A2:A83 --format dd/mm/yyyy   # force-convert text dates to real dates
xls format-set report.xlsx C2:C154 'dd/mm/yyyy'   # set a number format
xls autofit report.xlsx              # fit column widths to content
xls style report.xlsx A1:D1 --bold --bg FFFF00    # basic styling
xls copy report.xlsx A1:B3 D1        # copy a range to an anchor (move = copy+clear)
xls insert-row report.xlsx 3 -n 2    # insert 2 blank rows before row 3
xls delete-col report.xlsx C         # delete column C (letter or number)
xls add-sheet report.xlsx Summary    # add / delete / rename sheets
xls rename-sheet report.xlsx Sheet1 Data

# Safety flags on any mutating command
xls set report.xlsx A1 x --dry-run   # print the diff, write nothing
xls set report.xlsx A1 x --backup    # write report.xlsx.bak first
xls set report.xlsx A1 x --output copy.xlsx        # write a copy, leave the input

# Named ranges & Excel tables
xls table add report.xlsx A1:C20 --name Sales      # define a table over a range
xls get report.xlsx 'Sales[Amount]' --format csv   # read a table column
xls eval report.xlsx '=SUM(Sales[Amount])'         # structured references in formulas
xls name add report.xlsx TaxRate 'Sheet1!$E$1'     # define a named range
xls info report.xlsx                               # lists named ranges + tables

# Convert / import / interactive
xls export old.xls --format xlsx     # convert between formats
xls export report.xlsx -f csv -o -   # stream a text format to stdout
xls export huge.xlsx -f csv -o out.csv --stream    # memory-bounded export of huge files
xls import data.csv --into book.xlsx # add a CSV as a sheet
xls open book.xlsx                    # launch the interactive TUI

cat data.csv | xls eval - '=SUM(A:A)'              # read CSV from stdin with `-`
cat data.csv | xls set - A1 x --output out.csv     # stdin write needs --output

# Password-protected (encrypted) workbooks
xls info statement.xlsx              # without -p: reports the encryption scheme
xls get statement.xlsx B5 -p secret  # decrypt in-memory with --password / -p
```

`--password`/`-p` is a global flag accepted by any command. Without it, an
encrypted file is reported with its scheme (e.g. *ECMA-376 agile encryption:
AES-128-CBC, SHA1*) instead of opening. Mutating an encrypted file and saving
writes an **unencrypted** copy (re-encryption is not supported), and prints a
warning.

Run `xls --help` for the full reference and `xls <cmd> --help` per subcommand.

> **Numbers stored as text:** numeric functions (`SUM`, `AVERAGE`, `MIN`, …)
> coerce numeric-looking text (e.g. `6,000.00`, `1,51,302.63` from bank/CSV
> exports) at evaluation time, so such cells contribute without rewriting your
> data — the cells stay text and the file is untouched. `COUNT` stays strict
> (so `COUNT` < `COUNTA` flags text-stored numbers). To convert permanently,
> use `xls to-number <file> <range>` or the TUI `:tonum` on a selection.
>
> **Dates stored as text:** likewise, exports often store dates as text
> (`"04/04/2025"`). `xls to-date <file> <range> --format dd/mm/yyyy` parses them
> into real date serials and applies the format, so `get --raw --dates serial`
> (or `iso`) then emits machine-readable values. `xls profile` flags text-stored
> dates the same way it flags text-stored numbers.

## TUI

`xls open <file>` (or just `xls <file>`) launches a modal, vim-flavored editor:

- **Navigate** — arrows, `Tab`/`Enter`, `Ctrl+Arrow` (jump to data edge),
  `Home`/`End`, `Ctrl+Home`/`Ctrl+End`, `PgUp`/`PgDn`.
- **Select** — `Shift+Arrow` for ranges; mouse click & drag.
- **Edit** — type or `F2` to edit, `Esc` cancels, `Enter`/`Tab` commits and
  recalculates. Formula bar with live address.
- **Mouse** — click to select, drag to extend, wheel to scroll, click sheet tabs.
- **Clipboard** — `Ctrl+C`/`X`/`V` (internal ranges + system clipboard).
- **Undo/redo** — `Ctrl+Z` / `Ctrl+Y`.
- **Dialogs** — `Ctrl+G`/`F5` go-to, `Ctrl+F` find, `:` command palette,
  `Ctrl+S` save.

Virtual scrolling renders only visible cells, with frozen rows/columns and
number-format-aware display.

## Library

`default-features = false` gives a UI-free library exposing just `core`:

```toml
[dependencies]
xls = { version = "0.1", default-features = false }
```

```rust
use xls::core::{Workbook, Cell};
use xls::core::formula::Engine;

let mut wb = Workbook::new();
let s = wb.sheet_mut(0).unwrap();
s.set_a1("A1", Cell::Number(2.0));
s.set_a1("A2", Cell::Number(3.0));
s.set_a1("A3", Cell::Formula { expr: "=A1+A2".into(), cached: Default::default() });

Engine::new().recalc(&mut wb);
assert_eq!(wb.display_cell(0, 2, 0), "5");

xls::core::save_path(&wb, std::path::Path::new("out.xlsx")).unwrap();
```

## Formula coverage

All of Excel's standard worksheet functions are implemented one-by-one, grouped
by Microsoft category, each with known-answer tests (**all 522 implemented**).

| Category | Examples |
|----------|----------|
| Logical | `IF` `IFS` `SWITCH` `AND` `OR` `XOR` `IFERROR` |
| Math & Trig | `SUM` `SUMIFS` `SUMPRODUCT` `ROUND` `MOD` trig, `MDETERM` `MMULT` `SUBTOTAL` `AGGREGATE` |
| Statistical | `AVERAGEIFS` `MEDIAN` `STDEV.S` `PERCENTILE.INC` `RANK.EQ` `NORM.DIST` `CHISQ.TEST` `FREQUENCY` |
| Text | `LEFT` `MID` `SUBSTITUTE` `TEXT` `TEXTJOIN` `TEXTBEFORE` `REGEXEXTRACT` `TEXTSPLIT` |
| Lookup & Reference | `VLOOKUP` `XLOOKUP` `INDEX` `MATCH` `OFFSET` `INDIRECT` `XMATCH` |
| Dynamic arrays | `SORT` `SORTBY` `UNIQUE` `FILTER` `SEQUENCE` `VSTACK`/`HSTACK` `TAKE`/`DROP` (with cell **spill**) |
| LAMBDA | `LAMBDA` `LET` `MAP` `REDUCE` `SCAN` `BYROW`/`BYCOL` `MAKEARRAY` |
| Date & Time | `DATE` `EDATE` `EOMONTH` `NETWORKDAYS` `YEARFRAC` `WEEKNUM` |
| Financial | `PMT` `NPV` `IRR` `XIRR` `PRICE` `YIELD` `DURATION` `MIRR` |
| Engineering | base conversions, `BITAND`, `CONVERT`, `ERF`, complex `IM*` |
| Information | `ISNUMBER` `ISERROR` `N` `TYPE` `CELL` `SHEET` |
| Database | `DSUM` `DAVERAGE` `DGET` `DCOUNT` … |

Dynamic-array results **spill** into neighboring cells (with `#SPILL!` on
obstruction), and `LAMBDA` values are first-class (callable via `LET` and the
higher-order helpers). Operators broadcast element-wise over ranges/arrays
(`A1:A10>5`); scalar *functions* don't auto-map over an array argument — use
`MAP` for that (e.g. `MAP(SEQUENCE(10), LAMBDA(x, MOD(x,2)))`). **Stubbed** (documented `#N/A`): functions needing
external data, a host app, or a cube/OLAP connection (`WEBSERVICE`,
`STOCKHISTORY`, `CUBE*`, `RTD`, …).

## Format support

| Format | Read | Write | Notes |
|--------|:----:|:-----:|-------|
| XLSX (OOXML) | ✅ | ✅ | styles, shared strings, formulas, merges, frozen panes, named ranges, tables; opaque parts preserved; streaming reads for huge files |
| XLS (BIFF8) | ✅ | ✅ | the only pure-Rust BIFF8 **writer**; formulas stored as cached values |
| CSV / TSV | ✅ | ✅ | delimiter auto-detection, BOM handling, `encoding_rs` transcoding, type inference |

Password-protected files are decrypted with `--password` (ECMA-376 agile and
standard encryption — AES with SHA-1/256/384/512); legacy binary RC4/XOR and the
rare "extensible" scheme are detected and named but not decrypted.

## Building & testing

```sh
cargo test                          # full suite (CLI + TUI features)
cargo test --no-default-features    # core only — fast, no UI deps
cargo clippy --all-targets --all-features -- -D warnings
cargo bench                         # recalc + xlsx round-trip timings
cargo run --example gen_fixtures    # regenerate the binary test fixtures
```

The data model uses sparse storage (`BTreeMap` keyed by cell position), so a
sheet with a handful of cells in a million-row grid costs only those cells.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at
your option.
