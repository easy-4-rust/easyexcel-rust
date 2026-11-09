# Tasks: Dependency Security Remediation

Audit source: docs/audit/2026-11-08-dependency-security-audit.md
Baseline commit: dd673ea6a15909cc137d87eba26b7bedd43d323a
PR: #5

## Phase 0 — Audit and shared contract

- [x] T0.1 Capture the fixed audit baseline, all 9 vulnerability IDs and all 11 unsoundness IDs.
- [x] T0.2 Map confirmed dependency edges and manifest/adapter entry points; explicitly mark unresolved callsites.
- [x] T0.3 Define remediation order, MSRV boundary, feature preservation rules and serialized Cargo.lock ownership.
- [ ] T0.4 Add exact source line references for Calamine call sites and all Salvo test-feature consumers.
- [x] T0.5 Record exact CI audit commands, local tool versions, RustSec advisory-db commit and current Cargo.lock SHA-256; refresh these values after each dependency-group commit.

## Phase 1 — Actix

- [x] T1.1 Verify actix-web 4.18.2 and upstream actix-http 3.20.3 commit metadata, API compatibility and Rust 1.89 MSRV; pin upstream commit 61c6a15c until the crate release is available.
- [x] T1.2 Set Actix adapter/example and Web conformance MSRV to 1.89; keep core MSRV 1.88 and update the Linux CI split.
- [x] T1.3 Update Cargo.lock serially to actix-web 4.18.2 and patched actix-http 3.20.3; cargo tree confirms these versions replace 4.14.0/3.19.0.
- [x] T1.4 Add raw TCP malformed chunked trailer/pipelining regression for RUSTSEC-2026-0446; 1/1 passed.
- [x] T1.5 Add decoder output-limit and Brotli large-window regressions for RUSTSEC-2026-0448/0455/0447; 2/2 passed.
- [x] T1.6 Run Actix tests, Rust 1.89 adapter checks, targeted Rust 1.88 core checks, Clippy/fmt and re-audit. Actix advisories are absent; global audit remains blocked by Calamine, h2, zstd/zstd-safe and lru tasks.

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

- [ ] T6.1 CI cargo audit and local cargo audit --no-fetch --deny warnings both pass with 0 vulnerabilities, 0 unsound warnings and 0 yanked packages.
- [ ] T6.2 cargo deny check advisories bans licenses sources passes without ignore/allow additions.
- [ ] T6.3 cargo tree -i confirms none of the reported affected versions remain; review cargo tree -d.
- [ ] T6.4 Run workspace tests, adapter tests, Java golden tests, XLSX round-trip, benchmark build, fmt and Clippy.
- [ ] T6.5 Verify core Rust 1.88 and package-specific adapter MSRVs.
- [ ] T6.6 Update docs/audit report with final lockfile hash, RustSec DB revision, commit SHA, CI Run/Job IDs and test counts.
- [ ] T6.7 Mark PR #5 ready only after every task is complete and all required checks are green; merge only after final review.

## Evidence Log

| Task | Commit | Command / CI | Result | Evidence |
|---|---|---|---|---|
| T0.1–T0.3 | 6983857eec6c9f06764a4fd0c4086b05c8230c97 | Security Audit Run 42447661141 / Job 128629914044 | baseline failed: 9 vulnerabilities + 11 warnings | docs/audit + OpenSpec files |
| T0.5 | 6983857eec6c9f06764a4fd0c4086b05c8230c97 | cargo audit; cargo deny check advisories bans licenses sources; rustc/cargo 1.99.0; cargo-audit 0.22.2; cargo-deny 0.20.2 | captured | RustSec DB 309392b18f6b17022f82c701f300433126901068; Cargo.lock SHA256 435BB5E3285EBBCBC20628776264E662AB6BE22159684A5AE72930697E5D70CD |
| T1.1–T1.6 | 51d04e9b282a6a8b0f3ee6199179c3cdfcf10e17 | Actix security tests (3/3), conformance (5 passed/1 ignored), adapter unit tests (10/10), fmt, Clippy, Rust 1.88 core and Rust 1.89 adapter checks | Actix IDs 0446/0447/0448/0455 removed; global audit still has 5 vulnerabilities + 11 unsound warnings | Security Audit Run 42450269311 / Job 128637778367 failed on remaining groups; CI Run 42450269320: rustfmt, MSRV, Clippy, Doc and benchmark passed; OS tests/coverage pending |
