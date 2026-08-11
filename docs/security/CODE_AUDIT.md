# easyexcel-rust Code Audit Report

**Date:** 2026-08-11
**Scope:** Production code in `crates/*/src/` (excluding `#[cfg(test)]` modules)
**Auditor:** ZCode automated scan

---

## 1. Unsafe Code

### Findings: 0 actual `unsafe` blocks in production code

| Item | Location | Assessment |
|------|----------|------------|
| `add_merged_region_unsafe` (function name only) | `easyexcel-csv/src/stubs/sheet_stubs.rs:33` | **Not unsafe** -- function name contains "unsafe" but is a safe `const fn` no-op stub. No actual `unsafe` keyword. |
| `#![forbid(unsafe_code)]` | `easyexcel-web/src/lib.rs:6` | Positive: crate-level forbid |
| `#![deny(unsafe_code)]` | `easyexcel/src/core/mod.rs:8` | Positive: crate-level deny |

**No `#[allow(unsafe_code)]` overrides found.**

**Verdict:** ZERO unsafe blocks in production code. No FFI, no raw pointer manipulation, no manual memory management.

---

## 2. Clippy High-Risk Lints

### 2a. Direct Clippy run (pedantic, workspace-wide)

Total warnings: ~750 (mostly documentation). High-risk breakdown:

| Lint | Count | Severity |
|------|-------|----------|
| `cast_possible_truncation` | 12 (active warnings) + 20+ (suppressed via `#[allow]`) | **MEDIUM** |
| `cast_precision_loss` | 3 (active) + 2 (suppressed) | **LOW** |
| `cast_sign_loss` | 0 (active) + 3 (suppressed) | **LOW** |
| `missing_panics_doc` | 7 | **LOW** (documentation only) |
| `large_enum_variant` | 2 | **LOW** |
| Unsafe-related lints | 0 | **NONE** |
| `unwrap_used` / `expect_used` (clippy restriction) | Not enabled | N/A |

### 2b. Suppressed `cast_possible_truncation` hotspots

Key files with explicit `#[allow(clippy::cast_possible_truncation)]`:

- `easyexcel-model/src/model/dates.rs` (3 sites) -- date arithmetic, guarded by value ranges
- `easyexcel-xls/src/biff8/encode.rs` (6 sites) -- BIFF8 binary encoding, values bounded by format spec
- `easyexcel-xlsx/src/xlsx/reader.rs:582-596` -- pane parsing, clamped before cast

**Verdict:** Cast truncation sites are intentional and most are guarded by `.clamp()` or value-range validation. No unguarded truncation in critical paths.

---

## 3. Panic / Unwrap Risk Points

### 3a. `partial_cmp(...).unwrap()` on floats -- **HIGH RISK**

**8 instances** in formula statistics functions that will panic on NaN input:

| File | Line(s) | Function context |
|------|---------|-----------------|
| `mode_mult_to_harmean.rs` | 67, 85, 244, 280, 341, 381 | MODE, MODE.MULT, HARMEAN, etc. |
| `register_to_median.rs` | 631 | MEDIAN |
| `trimmean_to_beta_pdf.rs` | 14 | TRIMMEAN, BETA.DIST, etc. |

**Risk:** If a formula cell contains NaN (e.g., `=0/0`), these functions panic instead of returning `#NUM!`. The fix is `.partial_cmp(b).unwrap_or(Ordering::Equal)` (already used correctly in `coerce.rs:130`).

### 3b. `.expect()` on invariant assumptions -- MEDIUM RISK

| File | Line | Expression | Risk |
|------|------|-----------|------|
| `excel_math_context_precision_to_parse_long.rs` | 14, 262, 390, 397, 400, 408, 492, 508, 572 | Various parsing invariants | Low -- internal parser state, not user-controllable |
| `locale_generated.rs` | 15 | Locale decimal separator | Low -- hardcoded data |
| `excel_locale.rs:53` | `from_name("en_US")` | Default locale | Low -- compile-time constant |
| `data_formatter.rs:419` | `parse::<f64>().unwrap()` | Format roundtrip | Low -- controlled format string |
| `csv_workbook.rs:198` | Cell style append | Internal invariant | Low -- immediately after push |
| `csv_cell_style.rs:102,114` | Format metadata | Internal invariant | Low |

### 3c. `panic!` / `unreachable!` in production code

| File | Line | Context | Risk |
|------|------|---------|------|
| `registry.rs:51` | `unwrap_or_else(\|\| panic!("alias target not found"))` | Formula registry init | Low -- fails at startup if misconfigured |
| `is_arrayish_to_evaluator_impl.rs:84,112` | `unreachable!()` | Evaluator match arms | Low -- truly unreachable by type system |
| `read.rs:111` (derive) | `unreachable!("classify_primitive")` | Derive macro | Low -- exhaustive match guard |
| `write_charts.rs:435` | `panic!("invalid static BIFF8 hex payload")` | Chart writer | Low -- hardcoded data |
| `longest_match_column_width_style_strategy.rs:195` | `panic!("poison the cache")` | Test-only poison injection | Low |

### 3d. `NaiveDate::from_ymd_opt(...).unwrap()` -- MEDIUM RISK

Multiple instances in `received_to_pricemat.rs` and `register_to_datedif_fn.rs` constructing dates from parsed values. Invalid dates (e.g., month=13, day=32) would panic. These are formula functions processing user-supplied cell values.

**Verdict:** The `partial_cmp().unwrap()` on NaN is the most dangerous finding -- it can be triggered by spreadsheet formulas. The NaiveDate unwraps are also user-reachable. Other panic points are on internal invariants.

---

## 4. Boundary Safety

### 4a. ResourceLimits -- Defined but NOT enforced in XLSX reader

`ResourceLimits` struct (in `easyexcel-io/src/io/resource_limits.rs`) defines:
- `max_file_bytes`: 256 MB (default)
- `max_sheets`: 256
- `max_rows`: 2,000,000
- `max_formula_cells`: 500,000
- `max_output_bytes`: 256 MB
- `max_cell_chars`: 1 MB
- `max_columns`: 16,384

**Problem:** The XLSX reader (`reader.rs:read_zip`) reads ALL ZIP entries into memory without checking any limits:
```rust
for i in 0..archive.len() {
    let mut f = archive.by_index(i)?;
    // ... no size check against ResourceLimits
    f.read_to_end(&mut data)?;  // unbounded
    parts.insert(name, data);
}
```

The streaming reader (`stream.rs`) also does not enforce limits. `ResourceLimits` is only enforced in the Markdown import path (`markdown_import_executor.rs:42-45`).

### 4b. ZIP Bomb Protection -- **ABSENT**

- `reader.rs:94-98`: Uses `f.size()` (uncompressed size from ZIP header) for capacity pre-allocation but does NOT check against `max_file_bytes`.
- A crafted ZIP with inflated header sizes can trigger massive memory allocation before any data is read.
- No check on total decompressed size across all entries.

### 4c. XML Entity Expansion (XXE) -- **SAFE**

- Uses `quick_xml` Rust crate which is a pull parser.
- Does NOT process DTDs, does NOT expand external entities, does NOT resolve SYSTEM references.
- No `<!DOCTYPE>` or `<!ENTITY>` processing in any reader path.
- **Verdict:** Immune to XXE by architecture.

### 4d. Path Traversal -- **INCONSISTENT**

Two path normalization functions exist with different security postures:

| Function | Location | `..` handling | Security |
|----------|----------|--------------|----------|
| `normalize_path` | `package.rs:147-166` | Returns error if `..` escapes root | **SECURE** |
| `normalize_rel_path` | `reader.rs:269-288` | Silently pops base directory | **WEAK** |

`normalize_rel_path` is used in the main XLSX reader path (`reader.rs:160,188`). A malicious relationship target like `../../../../etc/passwd` would produce `etc/passwd` (after popping all base segments) rather than an error. In practice, this path is used to look up ZIP entries by name, so it cannot escape the ZIP archive. However, it is defense-in-depth weak.

### 4e. Formula Recursion Depth -- **SAFE**

`evaluator_impl.rs:34,471` enforces `MAX_DEPTH = 256` (defined in `binding_to_wants_reference.rs:18`), preventing stack overflow from circular formula references.

---

## 5. Additional Findings

### 5a. `with_checks(false)` on XML attribute parsing

`package.rs:84`: `element.attributes().with_checks(false)` disables well-formedness checking on attributes. This is a deliberate performance choice but means malformed XML attributes are silently accepted.

### 5b. `read_to_end` on encrypted payload

`crypto.rs:50`: `stream.read_to_end(&mut buf)?` reads the entire decrypted payload into memory. For password-protected files, the encrypted container size is bounded by the file, but the inner ZIP could still be a bomb.

---

## 6. Production Readiness Conclusion

| Category | Status | Verdict |
|----------|--------|---------|
| Unsafe code | 0 blocks | **PASS** |
| Clippy high-risk lints | 0 errors, 12 truncation warnings (guarded) | **PASS** |
| Panic on NaN in formula stats | 8 `partial_cmp().unwrap()` | **FAIL** -- user-triggerable |
| Panic on invalid dates | 5+ `from_ymd_opt().unwrap()` | **FAIL** -- user-triggerable |
| ResourceLimits enforcement | Defined but unused in XLSX reader | **FAIL** -- no protection |
| ZIP bomb protection | Absent | **FAIL** -- no protection |
| XXE protection | Safe by architecture (quick_xml) | **PASS** |
| Path traversal | Inconsistent normalization | **WARN** -- low real risk |
| Formula recursion | Bounded at depth 256 | **PASS** |

### Overall: **NOT production-ready for untrusted input**

The codebase is well-structured with zero unsafe code and good architectural decisions (quick_xml pull parser, formula depth limiting). However, three categories of issues block production readiness with untrusted input:

1. **8 `partial_cmp().unwrap()` in formula statistics** -- trivial fix, replace with `.unwrap_or(Ordering::Equal)`.
2. **5+ `NaiveDate::from_ymd_opt().unwrap()` in date formulas** -- replace with proper error propagation.
3. **No ResourceLimits enforcement in XLSX/ZIP reading** -- the limits infrastructure exists but is not wired into the reader pipeline. This enables memory exhaustion via crafted files.

For **trusted input** (internal tools, CI pipelines), the codebase is production-ready today.

---

*Report generated by automated code audit. No source files were modified.*
