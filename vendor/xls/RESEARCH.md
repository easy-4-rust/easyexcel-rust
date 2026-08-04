# xls — Research

## Project Goal

A Rust CLI + TUI spreadsheet application that reads/writes XLS (BIFF8), XLSX (OOXML), and CSV formats. Supports formulas, formatting, multi-sheet workbooks, and interactive editing with mouse support. Essentially "Microsoft Excel for the terminal."

---

## File Formats

### XLS (Binary Interchange File Format — BIFF8)

**Container:** OLE2 Compound File Binary (CFB). Mini filesystem with sectors (512 bytes), FAT chains, directory entries. The `Workbook` stream (or `Book` for older files) contains all BIFF records.

**Specs:** [MS-XLS](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-xls/cd03cb5f-ca02-4934-a391-bb674cb8aa06), [MS-CFB](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b), [OpenOffice reference PDF](https://www.openoffice.org/sc/excelfileformat.pdf)

**Record format:** 4-byte header (2-byte type + 2-byte length) followed by data. Max record data: 8,224 bytes; larger data split across CONTINUE records (type `0x003C`).

**BIFF versions:** BIFF2 through BIFF8. BIFF8 (Excel 97-2003) is the target — identified by BOF record `vers=0x0600`. Key additions: Unicode (UTF-16LE), Shared String Table, 65536 rows.

**Key record types:**

| Record | Hex | Purpose |
|--------|-----|---------|
| BOF | 0x0809 | Start of substream (globals dt=0x0005, sheet dt=0x0010) |
| EOF | 0x000A | End of substream |
| BOUNDSHEET | 0x0085 | Sheet name, type, visibility, stream offset |
| SST | 0x00FC | Shared String Table |
| LABELSST | 0x00FD | String cell (SST index) |
| NUMBER | 0x0203 | IEEE 754 double cell |
| RK | 0x027E | Compressed numeric cell (4-byte encoded) |
| MULRK | 0x00BD | Multiple consecutive RK cells |
| BOOLERR | 0x0205 | Boolean or error cell |
| FORMULA | 0x0006 | Formula cell with RPN token stream |
| BLANK | 0x0201 | Empty cell with formatting |
| MULBLANK | 0x00BE | Multiple consecutive blank cells |
| XF | 0x00E0 | Extended Format (cell style, 20 bytes) |
| FORMAT | 0x041E | Number format string |
| FONT | 0x0031 | Font record (index 4 skipped by convention) |
| DIMENSION | 0x0200 | Used range bounds |
| ROW | 0x0208 | Row properties |
| MERGECELLS | 0x00E5 | Merged cell ranges |
| DATEMODE | 0x0022 | 1900 vs 1904 date system |
| FILEPASS | 0x002F | Password protection flag |

**Cell storage:** All cell records share a 6-byte header: row (u16), col (u16), XF index (u16). NUMBER stores 8-byte IEEE 754 double. RK encodes a 4-byte compressed number (bit 0: divide by 100, bit 1: integer vs double-top-30-bits). LABELSST stores a 4-byte SST index.

**SST structure:** Header has total reference count + unique count, followed by XLUnicodeString entries. Each string: 2-byte char count, 1-byte grbit (compressed/UTF-16LE, rich text runs, phonetic data), then character bytes. CONTINUE records can split strings mid-character with a fresh grbit byte at each boundary.

**Formula storage:** RPN token stream (Ptg tokens). Key tokens: PtgRef (cell ref), PtgArea (range), PtgFunc/PtgFuncVar (function calls), PtgInt/PtgNum (constants), PtgStr (string literals), arithmetic operators. Three token classes: reference (0x2x), value (0x4x), array (0x6x).

**Formatting chain:** Cell → XF record (ifnt/ifmt/border/fill/alignment) → FONT record + FORMAT record. First 16 XFs are built-in styles. Built-in number format IDs 0-49 don't need FORMAT records; custom formats start at ID 164.

**Limits:** 65,536 rows, 256 columns, 32,767 chars per cell, 8,224 bytes per record, 56-color palette, 4,000 XF records, 30 function args, 7 nesting levels.

### XLSX (Office Open XML SpreadsheetML)

**Container:** ZIP archive following Open Packaging Conventions (OPC).

**Specs:** [ECMA-376](https://ecma-international.org/publications-and-standards/standards/ecma-376/) (free), ISO/IEC 29500 (paid, same content), [MS-XLSX](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-xlsx/)

**Directory layout:**
```
[Content_Types].xml          — MIME type registry
_rels/.rels                  — root relationships
xl/workbook.xml              — sheet list, defined names, calc settings
xl/sharedStrings.xml         — SST (deduplicated strings)
xl/styles.xml                — fonts, fills, borders, numFmts, cellXfs
xl/worksheets/sheet1.xml     — cell data per sheet
xl/worksheets/_rels/         — per-sheet relationships
xl/drawings/                 — charts, images
xl/theme/theme1.xml          — color/font theme
xl/calcChain.xml             — formula calc order (optional)
docProps/core.xml            — Dublin Core metadata
docProps/app.xml             — app metadata
```

**Primary namespace:** `http://schemas.openxmlformats.org/spreadsheetml/2006/main` (Transitional). Strict OOXML uses `http://purl.oclc.org/ooxml/spreadsheetml/main`. Parsers must handle both.

**Workbook XML:** `<sheets>` lists each sheet with name, sheetId, r:id (relationship to physical file). `date1904="1"` on `<workbookPr>` switches to Mac date epoch. `<definedNames>` stores named ranges.

**Cell XML:** Within `<sheetData>` → `<row r="1">` → `<c r="A1" t="s" s="2">`. Cell type (`t` attribute):

| t value | Meaning | `<v>` content |
|---------|---------|---------------|
| (absent/n) | Number | numeric string |
| s | Shared string | 0-based SST index |
| b | Boolean | 1 or 0 |
| e | Error | #DIV/0!, #N/A, etc. |
| str | Formula string result | the string itself |
| inlineStr | Inline string | value in `<is><t>` child |

**Formulas:** Stored as text in `<f>` element: `<f>SUM(A1:A5)</f>`. Shared formulas use `t="shared" si="0" ref="B2:B100"`. Array formulas use `t="array"`. Excel 2007+ functions prefixed with `_xlfn.`.

**Styles:** `styles.xml` contains numFmts, fonts, fills, borders, cellXfs (the main format table). Cell `s` attribute indexes into cellXfs. Built-in numFmt IDs 0-49; custom start at 164. Date detection: check numFmtId against known date format IDs (14-22, 45-47) or parse custom format for date tokens.

**Shared Strings:** `<si>` entries with `<t>` (plain) or `<r>` (rich text runs with `<rPr>` formatting). `xml:space="preserve"` required for whitespace preservation.

**Limits:** 1,048,576 rows (2^20), 16,384 columns (2^14, last=XFD), 32,767 chars per cell, 8,192 char formula length, 65,490 unique styles, 255 function args, 64 nesting levels.

### CSV

**No formal container** — plain text with delimiter-separated values.

**De facto standard:** RFC 4180. Fields separated by comma (or semicolon in European locales, tab for TSV). Quoted fields (`"..."`) can contain delimiters, newlines, and escaped quotes (`""`).

**Rust crate:** `csv` (BurntSushi). SAX-style streaming parser. `csv-core` does zero-alloc parsing. `ByteRecord` is ~30% faster than `StringRecord` (skips UTF-8 validation). Amortized record reuse (loop vs iterator) gives 2x speedup on large files.

**Edge cases:** UTF-8 BOM not auto-stripped (must handle manually). No encoding declaration (use `encoding_rs` for non-UTF-8). Excel uses semicolons in European locales. Flexible mode needed for ragged CSVs.

---

## Formula Engine

### Syntax

**A1 style** (default): letter columns + numeric rows. `$` makes absolute. Range `:`, union `,`, intersection ` ` (space). Cross-sheet: `Sheet1!A1`. 3D: `Sheet1:Sheet3!A1`.

**R1C1 style** (programmatic): `R2C3` absolute, `R[-1]C[2]` relative. Used internally for shared formula offset calculation.

### Parsing

Recursive descent with precedence climbing (Pratt parsing). Operator precedence (high to low): `:` → ` ` → `,` → unary `-` → `%` → `^` → `*,/` → `+,-` → `&` → comparisons.

AST node types: Number, Text, Bool, Error, CellRef, Range, BinaryOp, UnaryOp, FunctionCall, ArrayLiteral, ImplicitIntersection, NamedRange.

### Data Types & Coercion

All numbers are f64. TRUE/FALSE coerce to 1/0 in arithmetic. Numeric strings coerce to numbers for direct refs only. Empty cells → 0 (arithmetic) or "" (text). Type hierarchy for comparisons: numbers < text < booleans.

### Error Values

| Error | When |
|-------|------|
| #NULL! | Non-overlapping intersection |
| #DIV/0! | Division by zero/blank |
| #VALUE! | Wrong argument type |
| #REF! | Invalid cell reference |
| #NAME? | Unknown function/name |
| #NUM! | Invalid numeric result |
| #N/A | Lookup not found |

Errors propagate through all operations except IFERROR/IFNA/ISERROR.

### Functions (priority tiers for implementation)

**Tier 1 — Core (MVP):**
SUM, AVERAGE, MIN, MAX, COUNT, COUNTA, IF, AND, OR, NOT, CONCATENATE/CONCAT, LEN, LEFT, RIGHT, MID, TRIM, UPPER, LOWER, ROUND, ABS, MOD, INT, IFERROR, ISBLANK, ISERROR, ISNUMBER, ISTEXT, ROW, COLUMN, TODAY, NOW

**Tier 2 — Essential:**
VLOOKUP, HLOOKUP, INDEX, MATCH, SUMIF, SUMIFS, COUNTIF, COUNTIFS, AVERAGEIF, AVERAGEIFS, TEXT, VALUE, FIND, SEARCH, SUBSTITUTE, REPLACE, DATE, YEAR, MONTH, DAY, HOUR, MINUTE, SECOND, DATEDIF, EDATE, EOMONTH, POWER, SQRT, EXP, LN, LOG, LOG10, CEILING, FLOOR, SIGN, IF nested, IFS, SWITCH, LARGE, SMALL, RANK, MEDIAN, STDEV, VAR, TEXTJOIN, REPT, EXACT, T, N, TYPE, ROWS, COLUMNS, ADDRESS, INDIRECT, OFFSET, CHOOSE

**Tier 3 — Extended:**
XLOOKUP, XMATCH, FILTER, SORT, SORTBY, UNIQUE, SEQUENCE, PMT, FV, PV, NPV, IRR, RATE, NPER, SLN, SUMPRODUCT, SUBTOTAL, AGGREGATE, RAND, RANDBETWEEN, PI, SIN, COS, TAN, RADIANS, DEGREES, FACT, COMBIN, GCD, LCM, PERCENTILE, QUARTILE, CORREL, FORECAST, NORM.DIST, NORM.INV, NUMBERVALUE, CLEAN, CHAR, CODE, WEEKDAY, WEEKNUM, WORKDAY, NETWORKDAYS, DAYS360

### Dependency Graph & Recalculation

Build a DAG: formula cells → referenced cells. Forward map (cell → dependents) for dirty propagation, backward map (cell → precedents) for rebuild. Topological sort (Kahn's algorithm) for calc order.

On value change: mark dependents dirty transitively, recalculate only dirty cells in topo order. Volatile functions (NOW, TODAY, RAND, INDIRECT, OFFSET) always added to dirty set.

Circular reference detection via DFS 3-color marking. Optional iterative calculation mode (max iterations + convergence threshold).

### Array Formulas

Legacy CSE (Ctrl+Shift+Enter): fixed output range, stored as `t="array"`. Dynamic arrays (Excel 365): auto-spill, `#SPILL!` if blocked, spill range ref `A1#`. Implicit intersection (`@` operator) for legacy compatibility.

---

## TUI Design

### Framework: Ratatui + Crossterm

**Ratatui** — maintained fork of tui-rs, immediate-mode rendering with double-buffered cell-level diff. Only changed cells written to terminal each frame. **Crossterm** — cross-platform backend with full mouse support (SGR extended mode, no coordinate limits). Default and recommended pairing.

### Mouse Support

Crossterm's `EnableMouseCapture` activates SGR mouse mode automatically. Events: `MouseEventKind::Down/Up/Drag/Moved/ScrollUp/ScrollDown`. Coordinates are zero-based u16. Hit-testing: save rendered widget `Rect` positions during render, test mouse coords against them.

Filter `Moved` events (fire continuously) unless implementing hover. macOS quirk: Ctrl+click → right click.

### UI Components

1. **Cell grid** — custom windowed renderer (not built-in Table widget, which lacks horizontal scroll). Maintain `(scroll_row, scroll_col)` viewport. Render only visible cells. Frozen rows/cols via 4-region split (corner, top strip, left strip, main area).
2. **Column/row headers** — fixed row at top (A, B, C...), fixed column on left (1, 2, 3...). Scroll in sync with grid.
3. **Formula bar** — `tui-textarea` crate for editing. Shows cell address + formula/value.
4. **Sheet tabs** — bottom bar, `Tabs` widget or custom horizontal list.
5. **Status bar** — cell info, mode indicator (NORMAL/EDIT), file status.
6. **Command palette** — modal popup for commands/search.

### Editing UX

Modal approach (vim-style): Normal mode (navigate) → Edit mode (F2/Enter/typing). Cursor: arrows, Tab (confirm+right), Enter (confirm+down), Ctrl+arrows (jump to data edge), Home/End, PgUp/PgDn. Selection: Shift+arrows, Ctrl+Shift+arrows. Copy/paste via `arboard` crate for system clipboard.

### Performance

Virtual rendering: only draw visible cells regardless of sheet size. Ratatui's diff minimizes terminal output. Sparse storage (`HashMap<(row,col), Cell>`) for memory efficiency with large sheets. Lazy formula evaluation for non-visible cells.

### CLI vs TUI Mode

Detect with `std::io::IsTerminal`. CLI mode: clap subcommands for scripting (`get`, `set`, `export`, `import`, `eval`). TUI mode: interactive ratatui app.

### Undo/Redo

Command pattern with coarse-grained operations (SetCellValue, PasteRange, DeleteRows, FormatCells). `undo` crate provides Record (linear stack) with merge support for coalescing rapid edits. Ctrl+Z/Ctrl+Y keybindings.

### Architecture

**Single crate `xls`** (one publishable artifact — see Decision #8 in PLAN), organized as internal modules rather than a workspace so that `cargo install xls` / `cargo publish` ship exactly one crate:

- `core` — data model, formula engine, file I/O. Always compiled, no UI deps.
- `cli` — clap subcommands. Behind the `cli` feature (default on).
- `tui` — ratatui UI layer. Behind the `tui` feature (default on).
- `main.rs` — thin entry point that dispatches to CLI or TUI.
- `lib.rs` — re-exports `core` so the crate is usable as a library (`xls = { version, default-features = false }` pulls core only).

**Why not a Cargo workspace:** a binary crate cannot be published alone if it depends on local path crates — cargo requires every path dependency to also be published with a registry version. Modules + features give the same dependency-isolation benefit (core never pulls ratatui/clap) while keeping a single publishable crate.

### Reference Projects

- **IronCalc/TironCalc** — Rust + Ratatui + IronCalc engine, 300+ functions, xlsx support
- **sheetsui** — Ratatui + IronCalc, vim-style
- **cell-sheet-tui** — core/tui workspace split, formulas, CSV support
- **sc-im** — C, ncurses, gold standard TUI spreadsheet
- **visidata** — Python, large-data TUI explorer

---

## Existing Rust Crates

| Crate | Read | Write | Formats | Notes |
|-------|------|-------|---------|-------|
| calamine | Yes | No | xls/xlsx/xlsb/ods | Fast, MIT, no formatting/styles |
| rust_xlsxwriter | No | Yes | xlsx | Rich features, charts, formatting |
| umya-spreadsheet | Yes | Yes | xlsx/xlsm | Round-trip modify, slower reads |
| simple_excel_writer | No | Yes | xlsx | Minimal streaming writer |
| csv | Yes | Yes | csv | BurntSushi, fast, mature |
| cfb | Yes | Yes | OLE2 | Pure-Rust CFB container |
| ironcalc | — | — | formula engine | 300+ functions, xlsx I/O |
| quick-xml | — | — | XML parsing | SAX-style, fast, used by calamine |
| zip | — | — | ZIP | Read/write ZIP archives |

**Notable gap:** No Rust crate writes XLS/BIFF8. calamine is the only pure-Rust XLS reader.

---

## Date Handling

**1900 system** (default): serial 1 = Jan 1, 1900. Lotus 1-2-3 bug: serial 60 = Feb 29, 1900 (doesn't exist). Dates >= Mar 1, 1900: `NaiveDate(1899-12-30) + serial days`. Must replicate the bug for compatibility.

**1904 system** (Mac): serial 0 = Jan 1, 1904. Offset: 1462 days between systems.

**Times:** fractional part of serial. 0.5 = noon. Combined: date.time = integer.fraction.

**Detection:** numeric cell with date-pattern numFmt (built-in IDs 14-22, 45-47, or custom format with y/m/d/h/s tokens).

---

## Test Fixtures

### Priority Sources (permissive licenses)

| Source | License | Formats | Key Coverage |
|--------|---------|---------|--------------|
| calamine tests | MIT | xls/xlsx/xlsb/ods | dates, 1904 epoch, merges, formulas, password, OOM, BIFF5, Unicode |
| readxl tests | MIT | xls/xlsx pairs | 65536-row boundary, date rounding, UTF-8 sheet names, rich text, currency formats |
| xlrd samples | BSD | xls | corrupted files, invalid formulas, BIFF4, ragged rows, named ranges, German formats |
| XlsxWriter comparison | BSD | xlsx | 300+ feature-specific reference files, charts, conditional formatting |
| OpenOffice testdocs | Apache 2.0 | xls/xlsx/xlsb | BIFF2-12 coverage, formulas, charts, pivot, protection |
| sineemore/csv-test-data | — | csv | RFC 4180 edge cases with JSON ground truth |
| CharlesNepote/CSV-test-files | — | csv | encoding x delimiter x line-ending matrix |

### Edge Cases to Cover

- Multiple sheets, cross-sheet formulas
- All cell types (numbers, strings, dates, booleans, errors)
- Merged cells
- Rich text / formatting
- 65536-row XLS boundary, large XLSX files
- Unicode/international characters
- Password-protected (detection/error)
- Corrupted/malformed files (graceful error handling)
- CSV: BOM, different delimiters, quoted fields with newlines/commas, different line endings, encoding variants
