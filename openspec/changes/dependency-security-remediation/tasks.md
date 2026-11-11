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

- [x] T2.1 Locate calamine API calls; repository usage is in dev-dependencies or #[cfg(test)] code. Production Excel parsing uses easyexcel-xls / easyexcel-xlsx; source paths are recorded in the audit.
- [x] T2.2 Verify calamine 0.38.0 and Rust 1.88 build; upstream fix b8ec4f09cba6aea7d8359ababc240c9ed064751b.
- [x] T2.3 Upgrade Cargo.lock to calamine 0.38.0; EasyExcel reader API unchanged. Commit e4044f1505caac0413ecd4e8d4043e4d69c1f54e.
- [ ] T2.4 Short-directory and cyclic-DIFAT upstream CFB fixtures added and passed (2/2); distinct 64-byte mini-sector fixture remains pending.
- [ ] T2.5 Two hostile fixtures tested in isolated integration binary; OS-enforced memory-bound subprocess validation remains pending.
- [ ] T2.6 Partial: 2330 facade and 17 round-trip tests passed (one Windows-locale test excluded); Rust 1.88 check passed; audit now 4 vulnerabilities + 11 unsound warnings. Full Java Golden, XLSB and workspace CI pending.

## Phase 3 — h2

- [x] T3.1 Map both h2 0.3.27 paths: Actix HTTP 3.x optional h2 dependency and Rocket 0.5.1 -> rocket_http 0.5.1 -> Hyper 0.14.32.
- [x] T3.2 Confirm Actix HTTP 3.21.0 still pins h2 0.3.27; confirm Rocket 0.5.2 moves to Hyper 1.12/h2 0.4.20; record controlled Actix HTTP bridge decision D3-C in design.md.
- [x] T3.3 Upgrade Rocket/Rocket HTTP to 0.5.2 and vendor Actix HTTP 3.21.0 with a narrow HTTP 0.2 <-> http 1.5 bridge over h2 0.4.20; no HTTP/2 feature removed.
- [x] T3.4 Add an end-to-end Actix h2c regression using the h2 0.4 client; verifies HTTP/2 request parsing, response status/version/headers/body and flow-control release. The h2 0.4.20 upstream fix covers empty DATA frame flood; no duplicate local patch added.
- [x] T3.5 cargo tree -i h2 resolves only h2 0.4.20; cargo tree -i h2@0.3.27 returns no matching package; Actix HTTP/2 bridge test 1/1, Web conformance 41 tests, Actix adapter 10/10, cargo audit 0 vulnerabilities/0 unsound/0 yanked and cargo deny all pass.

## Phase 4 — zstd / zstd-safe

- [ ] T4.1 Verify zstd 0.13.4 + zstd-safe 7.4.0 semver compatibility and MSRV.
- [ ] T4.2 Verify whether Salvo test feature is required in production dependency scope; move/scope it only if tests remain complete.
- [ ] T4.3 Update zstd and zstd-safe lock graph serially; preserve compression and test capabilities.
- [ ] T4.4 Add cross-stream context reuse regression for RUSTSEC-2026-0391.
- [ ] T4.5 Add seekable invalid-offset and corrupt legacy-frame streaming regressions for RUSTSEC-2026-0398/0403.
- [ ] T4.6 Prove all 10 zstd-safe unsound advisories are removed by the locked version; run Salvo tests and audit.

## Phase 5 — ssfmt / lru

- [x] T5.1 Confirm ssfmt 0.1.2 requires lru ^0.16; upstream lru panic-safety fix is merged but lru 0.18 is semver-incompatible.
- [x] T5.2 Use a minimal vendored lru 0.16.4 security backport (vendor/lru/EASYEXCEL_PATCH.md) instead of forking ssfmt; record upstream PR #238 and merged commit f9a7f00fcf2d33e00adb03758cb350aaaa52cddb.
- [x] T5.3 Patch crates.io lru to vendor/lru 0.16.4 without changing package version/API or violating ssfmt semver; cargo tree confirms ssfmt uses the path-patched package.
- [x] T5.4 Panic-safety regression 1/1 and easyexcel-format tests 93/93 passed; cargo audit no longer reports RUSTSEC-2026-0253; cargo deny licenses/sources pass. Full Java Golden remains part of T6.4.

## Phase 6 — Final verification and closeout

- [x] T6.1 Local cargo audit --no-fetch passes with 0 vulnerabilities, 0 unsound warnings and 0 yanked packages; CI rerun is still required after push.
- [x] T6.2 cargo deny check advisories bans licenses sources passes without ignore/allow additions.
- [x] T6.3 cargo tree confirms only h2 0.4.20; h2 0.3.27 is absent; lru resolves to local patched 0.16.4; cargo tree -d shows only reviewed unrelated duplicates (winnow, zstd/zstd-safe maintenance lines).
- [ ] T6.4 Run workspace tests, adapter tests, Java golden tests, XLSX round-trip, benchmark build, fmt and Clippy.
- [x] T6.5 Targeted core Rust 1.88 check and Actix/Salvo/Web conformance Rust 1.89 checks pass.
- [x] T6.6 Update docs/audit report with current worktree SHA, Cargo.lock hash, RustSec DB revision, toolchain, test evidence and known host-specific blockers; refresh CI IDs after push.
- [ ] T6.7 Mark PR #5 ready only after every task is complete and all required checks are green; merge only after final review.

## Evidence Log

| Task | Commit | Command / CI | Result | Evidence |
|---|---|---|---|---|
| T0.1–T0.3 | 6983857eec6c9f06764a4fd0c4086b05c8230c97 | Security Audit Run 42447661141 / Job 128629914044 | baseline failed: 9 vulnerabilities + 11 warnings | docs/audit + OpenSpec files |
| T0.5 | 6983857eec6c9f06764a4fd0c4086b05c8230c97 | cargo audit; cargo deny check advisories bans licenses sources; rustc/cargo 1.99.0; cargo-audit 0.22.2; cargo-deny 0.20.2 | captured | RustSec DB 309392b18f6b17022f82c701f300433126901068; Cargo.lock SHA256 435BB5E3285EBBCBC20628776264E662AB6BE22159684A5AE72930697E5D70CD |
| T1.1–T1.6 | 51d04e9b282a6a8b0f3ee6199179c3cdfcf10e17 | Actix security tests (3/3), conformance (5 passed/1 ignored), adapter unit tests (10/10), fmt, Clippy, Rust 1.88 core and Rust 1.89 adapter checks | Actix IDs 0446/0447/0448/0455 removed; global audit still has 5 vulnerabilities + 11 unsound warnings | Security Audit Run 42450269311 / Job 128637778367 failed on remaining groups; CI Run 42450269320: rustfmt, MSRV, Clippy, Doc and benchmark passed; OS tests/coverage pending |
| T2.1–T2.3 (T2.4–T2.6 partial) | e4044f1505caac0413ecd4e8d4043e4d69c1f54e | calamine 0.38.0; 2 hostile CFB fixtures; Rust 1.88 check; facade 2330; round-trip 17 | RUSTSEC-2026-0436 absent; local audit 4 vulnerabilities + 11 unsound warnings | Upstream calamine b8ec4f0; CI pending |
| T3.1–T3.5 | worktree changes based on 1086ac8ad9537764aa9a6db36a208305f1bd599b | h2 0.4.20 only; Actix h2c bridge 1/1; web conformance 41 tests; Actix adapter 10/10; audit/deny pass | RUSTSEC-2026-0258 removed | Cargo.lock + vendor/actix-http + actix_h2.rs |
| T5.1–T5.4 | worktree changes based on 1086ac8ad9537764aa9a6db36a208305f1bd599b | vendored lru panic-safety regression 1/1; easyexcel-format 93/93; audit/deny pass | RUSTSEC-2026-0253 removed | Cargo.lock + vendor/lru |
| T6.1–T6.3/T6.5/T6.6 | current uncommitted worktree | cargo audit 0/0/0; cargo deny all pass; h2 0.3 absent; Rust 1.88/1.89 targeted checks pass | Security dependency gate locally green | docs/audit current verification snapshot |
