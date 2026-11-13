# Tasks: Dependency Security Remediation

Audit source: docs/audit/2026-11-08-dependency-security-audit.md
Baseline commit: dd673ea6a15909cc137d87eba26b7bedd43d323a
PR: #5

## Phase 0 — Audit and shared contract

- [x] T0.1 Capture the fixed audit baseline, all 9 vulnerability IDs and all 11 unsoundness IDs.
- [x] T0.2 Map confirmed dependency edges and manifest/adapter entry points; explicitly mark unresolved callsites.
- [x] T0.3 Define remediation order, MSRV boundary, feature preservation rules and serialized Cargo.lock ownership.
- [x] T0.4 Add exhaustive git grep-verified source references for Calamine manifests/callsites and the only Salvo test-feature consumer; see audit §10.1.1. Calamine remains dev/test-only; Salvo test is scoped to Web conformance dev-dependencies.
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
- [x] T2.4 Short-directory and cyclic-DIFAT upstream CFB fixtures plus a generated minimal CFB with exactly one 64-byte mini-sector are covered. The fixture is independently opened through `cfb::CompoundFile`; its `Workbook` stream is read and asserted to be exactly 64 bytes before invoking Calamine. Windows targeted CFB tests pass 3/3.
- [x] T2.5 Linux CI Run 43150254881 / Job 130957481545 executes all four CFB security tests successfully, including the Linux-only child-process regression. The parent launches the test executable through util-linux `prlimit --as=536870912` before the child parses the 512-byte cyclic-DIFAT fixture. Windows runs the three cross-platform CFB tests 3/3. The final implementation uses no unsafe Rust and no libc dev-dependency.
- [x] T2.6 GitHub Ubuntu workspace CI passes: EasyExcel facade 2331/2331, Java parity 95/95, Java Golden 88/88, round-trip 17/17; macOS workspace CI also passes. Rust 1.88 core and adapter MSRV checks pass. XLSB is explicitly unsupported by EasyExcel (CHANGELOG: XLSX/XLS/CSV only; no Xlsb code or .xlsb fixture), so XLSB compatibility is not claimed or treated as an acceptance requirement.

## Phase 3 — h2

- [x] T3.1 Map both h2 0.3.27 paths: Actix HTTP 3.x optional h2 dependency and Rocket 0.5.1 -> rocket_http 0.5.1 -> Hyper 0.14.32.
- [x] T3.2 Confirm Actix HTTP 3.21.0 still pins h2 0.3.27; confirm Rocket 0.5.2 moves to Hyper 1.12/h2 0.4.20; record controlled Actix HTTP bridge decision D3-C in design.md.
- [x] T3.3 Upgrade Rocket/Rocket HTTP to 0.5.2 and vendor Actix HTTP 3.21.0 with a narrow HTTP 0.2 <-> http 1.5 bridge over h2 0.4.20; no HTTP/2 feature removed.
- [x] T3.4 Verify the h2 0.4.16+ upstream fix and its upstream empty DATA-frame flood/backpressure regression; add an end-to-end Actix h2c bridge test with the h2 0.4 client covering request parsing, response status/version/headers/body and flow-control release. No duplicate local h2 patch.
- [x] T3.5 cargo tree -i h2 resolves only h2 0.4.20; cargo tree -i h2@0.3.27 returns no matching package; Actix HTTP/2 bridge test 1/1, Web conformance 41 tests, Actix adapter 10/10, cargo audit 0 vulnerabilities/0 unsound/0 yanked and cargo deny all pass.

## Phase 4 — zstd / zstd-safe

- [x] T4.1 Verify zstd 0.13.4 + zstd-safe 7.4.0 semver compatibility; both remain on their matching Salvo 0.13 dependency line and are Rust 1.88 compatible.
- [x] T4.2 Confirm Salvo TestClient is used only by tests/easyexcel-web-conformance/tests/salvo.rs; move Salvo's test feature from the workspace production dependency to the conformance crate's dev-dependency.
- [x] T4.3 Keep zstd 0.13.4/zstd-safe 7.4.0 for Salvo TestClient dev tests and zstd 0.14.3/zstd-safe 8.2.0 for Actix compression; no compression or test capability removed. Salvo test feature is no longer enabled in production adapters.
- [x] T4.4 No repository-owned zstd context API call exists; RUSTSEC-2026-0391 is fixed by zstd 0.13.4 upstream. Preserve Salvo TestClient compression feature only in dev tests and verify the adapter/conformance suites.
- [x] T4.5 No repository-owned zstd-safe seekable/legacy decoder API call exists; RUSTSEC-2026-0398/0403 are fixed by zstd-safe 7.4.0 upstream. Verify locked version and run Salvo plus web conformance tests.
- [x] T4.6 All 10 zstd-safe unsound advisories are absent from cargo audit; Salvo unit tests and web conformance pass; cargo audit/cargo deny pass.

## Phase 5 — ssfmt / lru

- [x] T5.1 Confirm ssfmt 0.1.2 requires lru ^0.16; upstream lru panic-safety fix is merged but lru 0.18 is semver-incompatible.
- [x] T5.2 Use a minimal vendored lru 0.16.4 security backport (vendor/lru/EASYEXCEL_PATCH.md) instead of forking ssfmt; record upstream PR #238 and merged commit f9a7f00fcf2d33e00adb03758cb350aaaa52cddb.
- [x] T5.3 Patch crates.io lru to vendor/lru 0.16.4 without changing package version/API or violating ssfmt semver; cargo tree confirms ssfmt uses the path-patched package.
- [x] T5.4 Panic-safety regression 1/1 and easyexcel-format tests 93/93 passed; cargo audit no longer reports RUSTSEC-2026-0253; cargo deny licenses/sources pass. Full Java Golden remains part of T6.4.

## Phase 6 — Final verification and closeout

- [x] T6.0 Follow-up CI detected newly issued RUSTSEC-2026-0486 for rustix 1.1.4; upgraded to rustix 1.1.6. A later fresh Security Audit detected RUSTSEC-2026-0528 for ref-swap 0.1.2 via Rocket 0.5.2; local lock update upgrades it to fixed 0.1.3. Both advisories post-date the original audit baseline.
- [x] T6.0.2 RUSTSEC-2026-0528: Rocket 0.5.2 resolves ref-swap 0.1.2; upgrade to 0.1.3 in commit 9f3988c2b1e76c8c04dabc69442fca9c893aae3b. Rocket adapter tests 8/8 and demo check pass. The 0.1.3 release makes RefSwap/OptionRefSwap invariant and changes OptionRefSwap::into_inner to return Option, removing the unsafe variance/null behavior.
- [x] T6.1 Fresh cargo audit --no-fetch --deny warnings passes against RustSec DB 4ef2df8e806a65e2f1ecc14f56f51249062b23d6 (1509 advisories), with 0 vulnerabilities, 0 unsound warnings and 0 yanked packages after rustix 1.1.6 and ref-swap 0.1.3. CI on 2a6c2d had reported RUSTSEC-2026-0528; the fix is committed in 9f3988c and awaits CI rerun.
- [x] T6.2 cargo deny check advisories bans licenses sources passes locally against the refreshed advisory DB after ref-swap 0.1.3; no ignore/allow/source exceptions added. GitHub Security Audit rerun for commit 9f3988c is pending.
- [x] T6.3 cargo tree confirms only h2 0.4.20; h2 0.3.27 is absent; lru resolves to local patched 0.16.4; Salvo zstd 0.13.4 is dev/test-only while Actix uses zstd 0.14.3. Duplicate zstd/zstd-safe versions are required by distinct upstream semver lines.
- [x] T6.4 GitHub CI Run 43354243228: Ubuntu + macOS workspace tests, coverage-pr, benchmark build, fmt, Clippy, docs and MSRV all passed on 67dab8005023b0befe5abe2b9684d5933600d75e. Ubuntu evidence includes EasyExcel 2331 tests, Java parity 95, Java Golden 88, round-trip 17 and all four CFB security tests including the prlimit child. Local Windows full-workspace run exceeded the 60-minute runner limit; focused CFB tests pass.
- [x] T6.5 Targeted core Rust 1.88 check and Actix/Salvo/Web conformance Rust 1.89 checks pass.
- [x] T6.6 Update docs/audit report with pushed commit 2a6c2d6159ad4a670eb9a1f38a307fbfa6f2c843 and local lock-fix commit 9f3988c2b1e76c8c04dabc69442fca9c893aae3b, Rustix/ref-swap versions, Cargo.lock SHA-256 192556FB425768E93FB15020E6F8DB1FDBCF19804F7FA7A75229B7440F931822, RustSec DB 4ef2df8e806a65e2f1ecc14f56f51249062b23d6, toolchain and CI evidence.
- [ ] T6.7 Mark PR #5 ready only after every task is complete and all required checks are green; merge only after final review.

## Evidence Log

| Task | Commit | Command / CI | Result | Evidence |
|---|---|---|---|---|
| T0.1–T0.3 | 6983857eec6c9f06764a4fd0c4086b05c8230c97 | Security Audit Run 42447661141 / Job 128629914044 | baseline failed: 9 vulnerabilities + 11 warnings | docs/audit + OpenSpec files |
| T0.5 | 6983857eec6c9f06764a4fd0c4086b05c8230c97 | cargo audit; cargo deny check advisories bans licenses sources; rustc/cargo 1.99.0; cargo-audit 0.22.2; cargo-deny 0.20.2 | captured | RustSec DB 309392b18f6b17022f82c701f300433126901068; Cargo.lock SHA256 435BB5E3285EBBCBC20628776264E662AB6BE22159684A5AE72930697E5D70CD |
| T1.1–T1.6 | 51d04e9b282a6a8b0f3ee6199179c3cdfcf10e17 | Actix security tests (3/3), conformance (5 passed/1 ignored), adapter unit tests (10/10), fmt, Clippy, Rust 1.88 core and Rust 1.89 adapter checks | Actix IDs 0446/0447/0448/0455 removed; global audit still has 5 vulnerabilities + 11 unsound warnings | Security Audit Run 42450269311 / Job 128637778367 failed on remaining groups; CI Run 42450269320: rustfmt, MSRV, Clippy, Doc and benchmark passed; OS tests/coverage pending |
| T2.1–T2.3 (T2.4–T2.6 partial) | e4044f1505caac0413ecd4e8d4043e4d69c1f54e | calamine 0.38.0; 2 hostile CFB fixtures; Rust 1.88 check; facade 2330; round-trip 17 | RUSTSEC-2026-0436 absent; local audit 4 vulnerabilities + 11 unsound warnings | Upstream calamine b8ec4f0; CI pending |
| T0.4 | 67dab8005023b0befe5abe2b9684d5933600d75e | git grep for Calamine source/manifest refs and Salvo TestClient/feature refs | Exact source references recorded; Calamine and Salvo test features are test-only | docs/audit §10.1.1 + tasks T0.4 |
| T2.4–T2.6 | 67dab8005023b0befe5abe2b9684d5933600d75e | CFB integration 4/4 on Ubuntu (including prlimit child); Windows CFB 3/3; EasyExcel 2331; Java parity 95; Java Golden 88; round-trip 17 | Calamine CFB acceptance tests pass; XLSB explicitly unsupported | CI Run 43354243228 / Job 131652325519 + local fixture validation |
| T3.1–T3.5 | a90b61dfad447f3be0a1ad8ff1f0ff737fe978b1 | h2 0.4.20 only; Actix h2c bridge 1/1; web conformance 41 tests; Actix adapter 10/10; audit/deny pass | RUSTSEC-2026-0258 removed | Cargo.lock + vendor/actix-http + actix_h2.rs |
| T4.1–T4.6 | local follow-up changes after a90b61d | Salvo TestClient 5 passed/1 ignored; Salvo adapter 6/6; web conformance 41 tests; production dependency tree has no zstd 0.13.4 path; audit/deny pass | All zstd/zstd-safe advisories and 10 unsound warnings absent | Cargo.toml + conformance dev-dependency feature scope |
| T6.0–T6.2 | 9f3988c2b1e76c8c04dabc69442fca9c893aae3b | rustix 1.1.4 -> 1.1.6; ref-swap 0.1.2 -> 0.1.3; fresh cargo audit 0/0/0; cargo deny all pass; Rocket tests 8/8; Rocket example check pass | RUSTSEC-2026-0486 and RUSTSEC-2026-0528 removed locally; CI rerun pending | Cargo.lock + RustSec DB 4ef2df8e806a65e2f1ecc14f56f51249062b23d6 |
| T5.1–T5.4 | a90b61dfad447f3be0a1ad8ff1f0ff737fe978b1 | vendored lru panic-safety regression 1/1; easyexcel-format 93/93; audit/deny pass | RUSTSEC-2026-0253 removed | Cargo.lock + vendor/lru |
| T6.1–T6.6 | local follow-up after 2a6c2d6 | strict audit/deny pass against fresh DB; h2 0.3 absent; rustix 1.1.6; ref-swap 0.1.3; Rocket tests 8/8; Actix/Rustls guard in pushed commit | Local security gate green; GitHub CI pending after ref-swap fix | docs/audit + CI Runs 43460022228 / 43460022248 |
