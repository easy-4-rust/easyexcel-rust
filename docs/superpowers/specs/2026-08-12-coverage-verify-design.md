# Test Coverage Verification Report

**Date:** 2026-08-12（更新）
**Tool:** cargo-llvm-cov (stable-aarch64-apple-darwin)
**Command:** `scripts/coverage.sh`（workspace 全量）
**数据来源：** `/tmp/cov10.json`（2026-08-12 17:51 生成）

> 注：本文件 2026-08-11 初始版本报告 75.53% 行覆盖率（旧值）。以下为 2026-08-12
> 覆盖率提升 6 轮后的最新实测数据。

---

## 1. Total Coverage Summary

| Metric | Covered | Total | Percentage | 变化（vs 08-11） |
|--------|---------|-------|------------|-----------------|
| **Lines** | 106,821 | 119,706 | **89.24%** | +13.71pp |
| **Regions** | 170,592 | 192,537 | **88.60%** | +21.91pp |
| **Functions** | 12,768 | 14,819 | **86.16%** | +8.02pp |

---

## 2. Gate Comparison

| Gate | Target | Actual | Status |
|------|--------|--------|--------|
| User requirement (lines) | >90% | 89.24% | **NEAR MISS** (-0.76pp) |
| User requirement (regions) | >90% | 88.60% | **NEAR MISS** (-1.40pp) |
| User requirement (functions) | >90% | 86.16% | **FAIL** (-3.84pp) |
| CI gate (`scripts/coverage.sh`) lines | >=95% | 89.24% | **FAIL** |
| CI gate (`scripts/coverage.sh`) regions | >=95% | 88.60% | **FAIL** |
| CI gate (`scripts/coverage.sh`) functions | >=95% | 86.16% | **FAIL** |

**结论：行覆盖率已从 75.53% 提升至 89.24%，距 90% 目标仅差 0.76pp。Region 和 Function 覆盖率也大幅改善。**

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
| **TOTAL** | **89.24%** | **88.60%** | **86.16%** | **119,706** |

> 注：上表 TOTAL 行已更新为 2026-08-12 实测值。各 crate 明细行仍为 2026-08-11 旧值，
> 仅供趋势参考，不再代表当前状态。行覆盖率从 75.53% 提升至 89.24%（+13.71pp）。

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

### RESULT: NEAR MISS（2026-08-12 更新）

| Criterion | 2026-08-11 旧值 | 2026-08-12 实测 | Status |
|-----------|----------------|----------------|--------|
| Lines coverage > 90% | 75.53% | **89.24%** | **NEAR MISS** (-0.76pp) |
| Regions coverage > 90% | 66.69% | **88.60%** | **NEAR MISS** (-1.40pp) |
| Functions coverage > 90% | 78.14% | **86.16%** | **FAIL** (-3.84pp) |
| CI gate (95%) | FAIL | FAIL | 仍需努力 |
| Test compilation | PARTIAL | **PASS**（easyexcel-test 已修复） | **改善** |
| Test pass rate | 99.6% (2122/2130) | **99.96%** (4447/4449) | **改善** |

> 2026-08-12 测量：`scripts/coverage.sh` → `/tmp/cov10.json`；`cargo test --workspace` →
> 4447 passed / 2 failed / 2 ignored（全 25+ 个测试二进制）。easyexcel lib 2197 全绿。

### 剩余 Recommendations

1. **行覆盖率从 89.24% 到 90%**：仅差 0.76pp，补充 ~900 行覆盖即可达标。
2. **Function 覆盖率 86.16%**：差距较大，需重点覆盖未调用的公共函数。
3. **0% 文件清理**：CSV stubs（628 行）、web adapter headers（77 行）仍为 0%。
4. **XLS 公式引擎**：`decode_formula_rpn.rs`（452 行 0%）需要工程投入。

### Honest Assessment（2026-08-12 更新）

行覆盖率已从 75.53% 提升至 89.24%（+13.71pp），easyexcel-test 编译问题已修复，
全 workspace 测试从 2122 passed 提升至 4447 passed。距 90% 目标仅差 0.76pp，
属"近在咫尺"状态。主要瓶颈从"测试基础设施损坏"转为"少量未覆盖模块"。

---

*Report initially generated 2026-08-11. Updated 2026-08-12 with fresh coverage and test measurements.*
*2026-08-12 data source: `/tmp/cov10.json`（Lines 89.24%）+ `cargo test --workspace`（4447 passed）*
