# Design: Dependency Security Remediation

## 1. Decision Authority

本 change 的唯一技术决策来源是本文件与审计报告：

- Audit baseline：docs/audit/2026-11-08-dependency-security-audit.md
- Baseline commit：dd673ea6a15909cc137d87eba26b7bedd43d323a
- PR：#5 / fix/compatible-security-deps-20261031

Agent 可以提出方案，但不能在未更新本文件前改变版本策略、MSRV、feature 边界、公共 API 或依赖来源。

## 2. Shared Architecture Constraints

1. Workspace root MSRV remains 1.88 unless an explicit ADR/change is approved. Framework adapters may have a higher package-specific MSRV.
2. Keep all existing framework integrations and HTTP/2/compression features.
3. Do not suppress advisories or unsound warnings.
4. Preserve ZIP 9.1.3 code/API migration.
5. Cargo.lock updates are serialized: one integration owner resolves the complete dependency graph and commits the lockfile.
6. Security fixes must be applied at the narrowest compatible dependency boundary; do not force incompatible versions into Cargo.lock.

## 3. Remediation Groups

### SEC-ACTIX

Target actix-web >=4.18.2 and actix-http >=3.20.3 to cover RUSTSEC-2026-0446, 0448, 0455 and 0447. The crates.io actix-http 3.20.3 release was not yet available at remediation time, so pin upstream actix/actix-web commit 61c6a15c1c0e505a5cd6842610a5f1faea432a83 through [patch.crates-io]; replace the git patch with the crates.io release once available. Actix Web 4.18.2 documents Rust 1.89+, so Actix adapter/example and Web conformance declare 1.89; core remains 1.88. This patch does NOT fix h2 0.3.27, which remains assigned to SEC-H2.

### SEC-CALAMINE

Target calamine >=0.38.0 for RUSTSEC-2026-0436. First locate all direct call sites and identify .xls/.xlsb/VBA entry paths. Preserve the public EasyExcel reader API and test malformed CFB/OLE2 inputs in a resource-limited subprocess. Regression coverage includes upstream short-directory and cyclic-DIFAT fixtures, a generated one-64-byte-mini-sector CFB workbook, and a Linux child process constrained by RLIMIT_AS=512 MiB while parsing the 512-byte cyclic-DIFAT fixture.

### SEC-H2

The vulnerable h2 0.3.27 is reached through actix-http and rocket_http -> hyper 0.14.32. Solve both paths. Prefer upstream compatible parent updates. If no upstream fix exists for the h2 0.3 line, prepare a narrowly scoped backport/fork of the upstream h2 fix with source review and regression tests. Do not disable HTTP/2 or leave one parent path unresolved.

Decision D3-A (2026-11-09): Rocket v0.5.2, published 2026-10-23, replaces rocket_http's Hyper 0.14.32 with Hyper 1.12.0 and directly declares h2 0.4.20. It declares MSRV 1.89. Upgrade Rocket adapter/demo only and move their MSRV checks to the Rust 1.89 group; keep root/core 1.88 and HTTP/2 enabled. Official source: https://github.com/rwf2/Rocket/blob/v0.5.2/core/http/Cargo.toml.

Decision D3-C (2026-11-12, implemented in pushed commit a90b61dfad447f3be0a1ad8ff1f0ff737fe978b1): Actix HTTP 3.21.0 still declares optional h2 0.3.27, and Actix Web 4.19.0 depends on Actix HTTP 3.21.0. To preserve Actix Web 4.x public APIs and HTTP/2, vendor the upstream Actix HTTP 3.21.0 source as a narrowly scoped local compatibility package at vendor/actix-http. Change only its internal HTTP/2 engine dependency to h2 0.4.20, add an http1 alias for http 1.5, and bridge request/response heads between Actix's public http 0.2 types and h2 0.4/http 1.x. Keep HTTP/1, HTTP/2, compression, TLS and WebSocket features. Rocket is upgraded to 0.5.2, whose rocket_http uses Hyper 1.12.0 and h2 0.4.20. The root workspace h2 dependency is 0.4.20. No h2 0.3 crate remains in Cargo.lock. The local Actix fork source provenance and compatibility boundary are documented in vendor/actix-http/EASYEXCEL_PATCH.md. The end-to-end h2c bridge test passes; the Actix HTTP standalone Rustls tests are blocked on the Windows host because OpenSSL development libraries are not installed, so Linux CI remains required.

### SEC-ZSTD

Use zstd 0.13.4 + zstd-safe 7.4.0 for Salvo 0.89.3's TestClient dev-only feature, and zstd 0.14.3 + zstd-safe 8.2.0 for Actix HTTP compression. The Salvo test feature is enabled only by tests/easyexcel-web-conformance dev-dependencies, not by production adapters; the root workspace Salvo dependency now enables only http1/server. The production dependency tree no longer contains the zstd 0.13 line. No repository-owned code directly calls zstd context, seekable or legacy streaming APIs; rely on the upstream patched versions for those internals and retain adapter/conformance regression tests. This closes RUSTSEC-2026-0391/0398/0403 and all 10 zstd-safe unsoundness advisories.

### SEC-LRU

ssfmt 0.1.2 constrains lru to ^0.16, while the panic-safety fix is in lru >=0.18.2. The upstream fix is merged in lru-rs PR #238 (commit f9a7f00fcf2d33e00adb03758cb350aaaa52cddb). To avoid an incompatible ssfmt upgrade, vendor lru 0.16.4 and backport only the panic-safe node-detach ordering; patch crates.io to vendor/lru without changing its version/API. The patch and regression are documented in vendor/lru/EASYEXCEL_PATCH.md.

## 4. Validation Strategy

Each remediation group must run:
- dependency graph inspection: cargo tree -i <crate> and cargo tree -d;
- targeted package tests and security regression tests;
- fmt, Clippy, package/workspace check;
- applicable MSRV check;
- cargo audit and cargo deny after lockfile changes.

Final gate requires zero vulnerabilities, zero unsound warnings, zero yanked packages, no new license/source violations, all target features retained, and complete workspace CI.

## 5. Agent Boundaries

- Investigator agents: read-only upstream/advisory research and source/callsite mapping; report exact URLs and evidence.
- Test agents: may add tests in their assigned group branch, but cannot edit Cargo.toml/Cargo.lock or change framework features.
- Dependency integrator: sole writer for workspace Cargo.toml, adapter manifests, Cargo.lock, deny.toml and final merge.
- Reviewer: verifies every changed version against advisory fixed range, MSRV, semver, feature graph and regression evidence.
- No agent may close a task based on cargo check alone.
