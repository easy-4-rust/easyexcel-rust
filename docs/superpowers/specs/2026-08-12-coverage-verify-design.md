# Test Coverage Verification Report

**Date:** 2026-08-11
**Tool:** cargo-llvm-cov (stable-aarch64-apple-darwin)
**Command:** `cargo llvm-cov --workspace --all-features --exclude easyexcel-test --exclude xtask --ignore-run-fail --ignore-filename-regex 'easyexcel-derive/src/lib.rs|easyexcel-reader/src/locale_generated.rs' --summary-only`

---

## 1. Total Coverage Summary

| Metric | Covered | Total | Percentage |
|--------|---------|-------|------------|
| **Lines** | 38,533 | 157,495 | **75.53%** |
| **Regions** | 8,020 | 12,025 | **66.69%** |
| **Functions** | 76,828 | 98,322 | **78.14%** |

---

## 2. Gate Comparison

| Gate | Target | Actual | Status |
|------|--------|--------|--------|
| User requirement (lines) | >90% | 75.53% | **FAIL** |
| User requirement (regions) | >90% | 66.69% | **FAIL** |
| User requirement (functions) | >90% | 78.14% | **FAIL** |
| CI gate (`scripts/coverage.sh`) lines | >=95% | 75.53% | **FAIL** |
| CI gate (`scripts/coverage.sh`) regions | >=95% | 66.69% | **FAIL** |
| CI gate (`scripts/coverage.sh`) functions | >=95% | 78.14% | **FAIL** |

**Conclusion: Coverage is 14-28 percentage points below the 90% target across all metrics.**

---

## 3. Per-Crate Breakdown (sorted by lines%, ascending)

| Crate | Lines% | Regions% | Funcs% | Total Lines |
|-------|--------|----------|--------|-------------|
| easyexcel-csv | 40.88% | 18.44% | 45.73% | 2,637 |
| easyexcel-formula | 64.81% | 72.38% | 64.18% | 19,132 |
| easyexcel-xlsx | 68.21% | 63.82% | 69.55% | 14,712 |
| easyexcel-salvo | 68.97% | 54.55% | 72.73% | 145 |
| easyexcel-actix | 71.58% | 56.00% | 73.28% | 190 |
| easyexcel-xls | 71.90% | 68.19% | 71.26% | 19,177 |
| easyexcel-derive | 72.27% | 75.61% | 78.81% | 1,980 |
| easyexcel-rocket | 73.83% | 64.71% | 80.77% | 107 |
| easyexcel-web | 73.87% | 72.95% | 74.59% | 1,018 |
| easyexcel-tabular | 76.20% | 56.60% | 72.18% | 685 |
| easyexcel-model | 76.24% | 78.51% | 75.60% | 3,178 |
| easyexcel-markdown | 77.28% | 77.48% | 77.74% | 1,241 |
| easyexcel-poem | 78.40% | 60.87% | 80.95% | 213 |
| easyexcel (main) | 81.14% | 68.25% | 84.46% | 86,527 |
| easyexcel-io | 81.69% | 85.20% | 86.31% | 2,125 |
| easyexcel-format | 83.75% | 76.27% | 82.54% | 1,963 |
| easyexcel-axum | 84.57% | 57.14% | 83.50% | 175 |
| easyexcel-utils | 85.73% | 89.33% | 80.27% | 771 |
| easyexcel-hyper | 87.70% | 66.67% | 90.91% | 187 |
| easyexcel-warp | 88.76% | 78.95% | 94.35% | 169 |
| easyexcel-cache | 88.92% | 85.94% | 88.06% | 424 |
| **TOTAL** | **75.53%** | **66.69%** | **78.14%** | **157,495** |

**No crate reaches 90% lines coverage.** The best is easyexcel-cache at 88.92%.

---

## 4. Low-Coverage Files (Top 20)

| File | Lines% | Regions% | Funcs% | Total | Missed |
|------|--------|----------|--------|-------|--------|
| easyexcel-actix/src/headers.rs | 0.00% | 0.00% | 0.00% | 26 | 26 |
| easyexcel-csv/src/csv/csv_cell/csv_cell_value.rs | 0.00% | 0.00% | 0.00% | 43 | 43 |
| easyexcel-csv/src/stubs/cell_stubs.rs | 0.00% | 0.00% | 0.00% | 26 | 26 |
| easyexcel-csv/src/stubs/cell_style_stubs.rs | 0.00% | 0.00% | 0.00% | 201 | 201 |
| easyexcel-csv/src/stubs/sheet_stubs.rs | 0.00% | 0.00% | 0.00% | 282 | 282 |
| easyexcel-csv/src/stubs/workbook_stubs.rs | 0.00% | 0.00% | 0.00% | 119 | 119 |
| easyexcel-derive/src/annotation/conditional_parser.rs | 0.00% | 0.00% | 0.00% | 70 | 70 |
| easyexcel-derive/src/annotation/data_validation_parser.rs | 0.00% | 0.00% | 0.00% | 89 | 89 |
| easyexcel-io/src/io/easy_excel_temp_file_creation_strategy.rs | 0.00% | 0.00% | 0.00% | 68 | 68 |
| easyexcel-io/src/io/media_type.rs | 0.00% | 0.00% | 0.00% | 38 | 38 |
| easyexcel-model/src/model/styles/color.rs | 0.00% | 0.00% | 0.00% | 3 | 3 |
| easyexcel-model/src/model/workbook/cell_to_workbook_impl/workbook.rs | 0.00% | 0.00% | 0.00% | 3 | 3 |
| easyexcel-poem/src/headers.rs | 0.00% | 0.00% | 0.00% | 19 | 19 |
| easyexcel-rocket/src/headers.rs | 0.00% | 0.00% | 0.00% | 13 | 13 |
| easyexcel-salvo/src/headers.rs | 0.00% | 0.00% | 0.00% | 19 | 19 |
| easyexcel-utils/src/utils/position_utils.rs | 0.00% | 0.00% | 0.00% | 60 | 60 |
| easyexcel-xls/src/biff8/protection.rs | 0.00% | 0.00% | 0.00% | 18 | 18 |
| easyexcel-xls/src/biff8/ptg/decode_formula_rpn.rs | 0.00% | 0.00% | 0.00% | 452 | 452 |
| easyexcel-xls/src/biff8/workbook/biff8cell_to_write_bof/biff8_hyperlink_kind.rs | 0.00% | 0.00% | 0.00% | 7 | 7 |
| easyexcel-xlsx/src/xlsx/event_reader/.../xlsx_display_options.rs | 0.00% | 0.00% | 0.00% | 3 | 3 |

---

## 5. Root Cause Analysis

### 5a. Coverage Distribution (853 files)

| Bucket | Files |
|--------|-------|
| 0% | 90 |
| 1-50% | 73 |
| 50-80% | 147 |
| 80-90% | 124 |
| 90-95% | 115 |
| 95-100% | 304 |

### 5b. Key Reasons for Low Coverage

1. **Stub files (easyexcel-csv):** 628 lines of stub implementations at 0%. CSV format is partially implemented with stub trait implementations that are never exercised.

2. **Unimplemented modules:** Files like `decode_formula_rpn.rs` (452 lines), `conditional_parser.rs` (70 lines), `data_validation_parser.rs` (89 lines) have 0% coverage -- these are either unfinished features or complex modules without dedicated tests.

3. **Web framework adapters:** `easyexcel-actix`, `easyexcel-poem`, `easyexcel-rocket`, `easyexcel-salvo` all have headers.rs at 0%. These are thin HTTP header integration layers that require runtime testing.

4. **Large core crate (easyexcel):** 86,527 lines at 81.14% -- the main crate is the biggest contributor to the gap. The XLS adapter (`template.rs` at 27.63%, `style.rs` at 50.67%) and `write_backend_selection.rs` (0%) are the worst files.

5. **XLS format crate:** `easyexcel-xls` at 71.90% -- BIFF8 formula encoding (`decode_formula_rpn.rs` at 0%) and roundtrip tests are failing, suggesting incomplete or broken formula support.

---

## 6. Test Execution Summary

| Metric | Value |
|--------|-------|
| Total tests passed | ~2,122 |
| Total tests failed | 8 |
| Compilation failures (easyexcel-test) | 6+ test files (API mismatch) |

### Failing Tests

| Crate | Test | Failure Reason |
|-------|------|----------------|
| easyexcel | `default_registry_preserves_formatted_numeric_text_for_string_fields` | Formatting: `"24.2"` vs `"24.20"` |
| easyexcel-format | `builtin_format_code_falls_back_to_cn_table` | Format resolution |
| easyexcel-format | `get_builtin_format_resolves_tables` | Format resolution |
| easyexcel-format | `format_raw_cell_contents_stub_returns_none` | Stub behavior |
| easyexcel-xls | `roundtrip_via_writer` | BIFF8 formula encoding failure |
| easyexcel-xls | `roundtrip_multisheet_custom_format_and_string_formula` | BIFF8 formula encoding failure |
| easyexcel-xls | `roundtrip_date_systems_and_format` | Date format encoding corruption |
| easyexcel-xls | `multiple_generated_charts_and_comments_emit_independent_drawing_groups` | Chart drawing group count mismatch |

### Integration Test Crate (easyexcel-test)

The entire `easyexcel-test` crate fails to compile due to API mismatches (`E0308: mismatched types`, `E0425`, `E0432`). Tests reference types/signatures that have changed. This crate was excluded from coverage measurement.

---

## 7. Production Readiness Verdict

### RESULT: NOT PRODUCTION-READY for coverage target

| Criterion | Status |
|-----------|--------|
| Lines coverage > 90% | **FAIL** (75.53%, gap: -14.47pp) |
| Regions coverage > 90% | **FAIL** (66.69%, gap: -23.31pp) |
| Functions coverage > 90% | **FAIL** (78.14%, gap: -11.86pp) |
| CI gate (95%) | **FAIL** (all metrics) |
| Test compilation | **PARTIAL** (easyexcel-test broken) |
| Test pass rate | **99.6%** (2122/2130 passed) |

### Recommendations to Reach 90%

1. **Fix easyexcel-test compilation** (biggest quick win): 6+ integration test files have API mismatches. Fixing these would add significant coverage from integration tests that currently cannot run.

2. **Easy targets (0% files with small line counts):** Headers.rs files across web adapters (~77 lines total), `position_utils.rs` (60 lines), `media_type.rs` (38 lines) -- these could be covered with basic unit tests.

3. **Stub coverage:** The CSV stubs (628 lines) and web adapter stubs represent intentional "not yet implemented" code. Either mark them as excluded from coverage or add minimal smoke tests.

4. **XLS formula engine:** `decode_formula_rpn.rs` (452 lines at 0%) and the failing roundtrip tests suggest an incomplete BIFF8 formula implementation. This needs engineering work, not just test additions.

5. **XLS adapter template:** `template.rs` (1,057 lines, 27.63% coverage) is the single largest uncovered file. Dedicated template-matching tests would boost the main crate significantly.

### Honest Assessment

The project has substantial test infrastructure (2,122+ passing tests) but coverage falls well short of 90%. The gap is not merely missing edge-case tests -- it reflects genuinely untested modules (stubs, formula engine, web adapters) and broken integration tests. Reaching 90% would require both fixing the broken test infrastructure AND adding tests for the identified low-coverage modules.

---

*Report generated by coverage verification agent on 2026-08-11. This is a verification-only report; no source code or tests were modified.*
