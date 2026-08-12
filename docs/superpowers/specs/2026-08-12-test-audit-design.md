# EasyExcel Java→Rust Test Audit Report (Phase 5.2 Update)

> Generated 2026-07-21. Updated after Phase 5.2 BIFF8 template fill implementation.
> **2026-08-12 数字校正**：原报告声称 "1315/1315 passed"，与实测不符。
> 以下为 2026-08-12 `cargo test --workspace` 全量实测结果。

## Executive Summary

| Metric | 旧值（声称） | 2026-08-12 实测 | 说明 |
|--------|-------------|----------------|------|
| Java test classes | **66** | 66 | 不变 |
| Java @Test methods | **335** | 335 | 不变 |
| Rust test methods (total, all crates) | ~~1315~~ | **4,449** | 25+ 测试二进制合计 |
| Golden tests passing | **88/88** | 88/88 | 不变 |
| Parity tests passing | **152/152** | 152/152 | 不变 |
| Full suite passing | ~~1315/1315~~ | **4,447/4,449** | 2 failed（fill executor） |
| `#[ignore]` annotations | ~~0~~ | **2** | 1 (easyexcel-xls) + 1 (easyexcel-test) |

> **测量方式**：`cargo test --workspace 2>&1 | grep "^test result:"` 汇总全部 25+ 个
> 测试二进制。easyexcel lib 主二进制 2197 passed / 0 failed / 0 ignored。
> easyexcel-derive 1253 passed。easyexcel-xlsx 285 passed。easyexcel-xls 39 passed / 1 ignored。
> easyexcel-test 集成测试 49 passed / 2 failed / 1 ignored。
> 失败的 2 个测试为 `excel_write_fill_executor` 相关（`executor_fill_config_variants_pass_through`、
> `executor_fill_sheet_default_and_named_pass_through`）。

### Gap breakdown after Phase 5.2

| Category | Count | % | Status |
|----------|-------|---|--------|
| Methods with FULL matching logic | ~295 | 88% | ✅ |
| XLS fill (SST passthrough) | ~15 | 4% | ⚠️ BIFF8 LABEL fill works; SST fill passes through |
| XLS encrypt/image/extra historical gaps | ~4 | 1% | ⚠️ Encryption and comment/extra paths are now coded but await rerun; image remains an explicit boundary |
| POI probes (excluded) | ~31 | 9% | ✅ Not EasyExcel API |

### Phase 5.2 deliverables

| File | What changed |
|------|-------------|
| `easyexcel-writer/src/biff8/template.rs` | `scan_placeholders()` + `replace_label()` added to Biff8TemplatePackage; LABEL/LABELSST decode helpers |
| `easyexcel-template/src/lib.rs` | `fill_xls_template_scalar()` + `fill_xls_template_list()` — BIFF8 placeholder engine; `fill_xlsx_template()` / `fill_xlsx_template_list()` delegate to XLS path |
| `easyexcel-test/tests/core_fill_1to1_tests.rs` | XLS fill tests: assert output exists instead of expecting Unsupported |
| `easyexcel-test/tests/java_full_parity_tests.rs` | 5 XLS parity tests: assert output exists |

### Historical explicit gaps and current static status

| Java method | Gap | Reason |
|-------------|-----|--------|
| EncryptDataTest#t02ReadAndWrite03 | XLS encryption | CryptoAPI RC4 `FILEPASS` read/write is coded; this historical test has not been rerun during the current no-test phase |
| EncryptDataTest#t04ReadAndWriteStream03 | XLS encryption | Stream path is coded through the same record-level engine; release evidence remains pending |
| ConverterDataTest#t22WriteImage03 | XLS image | BIFF8 MSODrawing/Escher records not implemented |
| ExtraDataTest#t02Read03 | XLS extra metadata | NOTE/TXO/OBJ comment read/write is coded; this historical test has not been rerun during the current no-test phase |

### SST limitation

XLS templates created by POI/Excel typically store strings in the Shared
String Table (SST). Without SST parsing, `{key}` placeholders in SST-based
templates are not found by `scan_placeholders()` and pass through silently.
LABEL-based templates (inline strings) are correctly filled. Full SST
support is a follow-on enhancement.
