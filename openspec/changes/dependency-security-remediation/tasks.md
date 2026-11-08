# Tasks: Dependency Security Remediation

Audit source: docs/audit/2026-11-08-dependency-security-audit.md
Baseline commit: dd673ea6a15909cc137d87eba26b7bedd43d323a
PR: #5

## Phase 0 — Audit and shared contract

- [x] T0.1 Capture the fixed audit baseline, all 9 vulnerability IDs and all 11 unsoundness IDs.
- [x] T0.2 Map confirmed dependency edges and manifest/adapter entry points; explicitly mark unresolved callsites.
- [x] T0.3 Define remediation order, MSRV boundary, feature preservation rules and serialized Cargo.lock ownership.
- [ ] T0.4 Add exact source line references for Calamine call sites and all Salvo test-feature consumers.
- [ ] T0.5 Record exact CI audit command, tool versions, RustSec database commit and Cargo.lock SHA-256.

## Phase 1 — Actix

- [ ] T1.1 Verify actix-web >=4.18.2 and actix-http >=3.20.3 release metadata, API compatibility and Rust 1.89 MSRV.
- [ ] T1.2 Update only the required Actix adapter/example MSRV; keep core MSRV 1.88.
- [ ] T1.3 Update the lockfile serially; prove no vulnerable actix-http/actix-web version remains.
- [ ] T1.4 Add malformed chunked trailer regression for RUSTSEC-2026-0446.
- [ ] T1.5 Add compressed-body and Brotli large-window resource-limit regressions for RUSTSEC-2026-0448/0455/0447.
- [ ] T1.6 Run Actix tests, Rust 1.89 adapter check, Rust 1.88 core check, Clippy/fmt and audit.

## Phase 2 — Calamine

- [ ] T2.1 Locate every calamine API call and map untrusted .xls/.xlsb/VBA input paths.
- [ ] T2.2 Verify calamine >=0.38.0 release/MSRV/API changes.
- [ ] T2.3 Upgrade and update lockfile; preserve EasyExcel reader public API.
- [ ] T2.4 Add truncated CFB directory, 64-byte mini-sector and cyclic DIFAT fixtures.
- [ ] T2.5 Execute hostile fixtures in a bounded subprocess; verify typed error/no panic/no unbounded allocation.
- [ ] T2.6 Run Java golden, XLS/XLSB import, workspace tests and audit.

## Phase 3 — h2

- [ ] T3.1 Map exact h2 0.3.27 paths from actix-http and rocket_http -> hyper 0.14.32.
- [ ] T3.2 Check upstream parent releases and h2 0.3 backport availability; record decision in design.md before editing.
- [ ] T3.3 Implement a compatible upstream fix/backport or update both parent chains; no HTTP/2 feature removal.
- [ ] T3.4 Add empty DATA frame flood/backpressure regression for Actix and Rocket paths.
- [ ] T3.5 Prove cargo tree -i h2 has no vulnerable 0.3.27 path; run adapter tests and audit.

## Phase 4 — zstd / zstd-safe

- [ ] T4.1 Verify zstd 0.13.4 + zstd-safe 7.4.0 semver compatibility and MSRV.
- [ ] T4.2 Verify whether Salvo test feature is required in production dependency scope; move/scope it only if tests remain complete.
- [ ] T4.3 Update zstd and zstd-safe lock graph serially; preserve compression and test capabilities.
- [ ] T4.4 Add cross-stream context reuse regression for RUSTSEC-2026-0391.
- [ ] T4.5 Add seekable invalid-offset and corrupt legacy-frame streaming regressions for RUSTSEC-2026-0398/0403.
- [ ] T4.6 Prove all 10 zstd-safe unsound advisories are removed by the locked version; run Salvo tests and audit.

## Phase 5 — ssfmt / lru

- [ ] T5.1 Verify whether ssfmt has a release allowing lru >=0.18.2.
- [ ] T5.2 If no release exists, prepare minimal ssfmt fork/patch; document source, revision and maintenance owner.
- [ ] T5.3 Upgrade lru without violating semver; preserve formatter API and cache behavior.
- [ ] T5.4 Run number/date format Java golden tests, cache tests, MSRV, licenses and audit.

## Phase 6 — Final verification and closeout

- [ ] T6.1 cargo audit --no-fetch --deny warnings passes with 0 vulnerabilities, 0 unsound warnings and 0 yanked packages.
- [ ] T6.2 cargo deny check advisories bans licenses sources passes without ignore/allow additions.
- [ ] T6.3 cargo tree -i confirms none of the reported affected versions remain; review cargo tree -d.
- [ ] T6.4 Run workspace tests, adapter tests, Java golden tests, XLSX round-trip, benchmark build, fmt and Clippy.
- [ ] T6.5 Verify core Rust 1.88 and package-specific adapter MSRVs.
- [ ] T6.6 Update docs/audit report with final lockfile hash, RustSec DB revision, commit SHA, CI Run/Job IDs and test counts.
- [ ] T6.7 Mark PR #5 ready only after every task is complete and all required checks are green; merge only after final review.

## Evidence Log

| Task | Commit | Command / CI | Result | Evidence |
|---|---|---|---|---|
| T0.1–T0.3 | pending docs commit | Security Audit Run 42447661141 / Job 128629914044 | baseline failed: 9 vulnerabilities + 11 warnings | GitHub Actions log |
