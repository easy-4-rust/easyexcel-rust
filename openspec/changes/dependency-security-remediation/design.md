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

Target calamine >=0.38.0 for RUSTSEC-2026-0436. First locate all direct call sites and identify .xls/.xlsb/VBA entry paths. Preserve the public EasyExcel reader API and test malformed CFB/OLE2 inputs in a resource-limited subprocess.

### SEC-H2

The vulnerable h2 0.3.27 is reached through actix-http and rocket_http -> hyper 0.14.32. Solve both paths. Prefer upstream compatible parent updates. If no upstream fix exists for the h2 0.3 line, prepare a narrowly scoped backport/fork of the upstream h2 fix with source review and regression tests. Do not disable HTTP/2 or leave one parent path unresolved.

Decision D3-A (2026-11-09): Rocket v0.5.2, published 2026-10-23, replaces rocket_http's Hyper 0.14.32 with Hyper 1.12.0 and directly declares h2 0.4.20. It declares MSRV 1.89. Upgrade Rocket adapter/demo only and move their MSRV checks to the Rust 1.89 group; keep root/core 1.88 and HTTP/2 enabled. Official source: https://github.com/rwf2/Rocket/blob/v0.5.2/core/http/Cargo.toml.

Decision D3-B (pending): actix-http v3.21.0 and v3.20.5 still declare optional h2 ^0.3.27 (https://github.com/actix/actix-web/blob/actix-http-v3.21.0/actix-http/Cargo.toml). Do not disable HTTP/2, suppress RUSTSEC-2026-0258, or claim the issue fixed by upgrading Rocket alone. Assess a controlled backport of https://github.com/hyperium/h2/pull/935 or an upstream Actix migration, including Cargo semver and security scanner implications, before any second-path changes.

### SEC-ZSTD

Prefer a compatible zstd 0.13 maintenance line (>=0.13.4) plus zstd-safe >=7.4.0 if upstream semver allows it; otherwise evaluate zstd 0.14.3 / zstd-safe >=8.2.0 as a separate API/MSRV decision. This group must close RUSTSEC-2026-0391, 0398, 0403 and all 10 unsoundness warnings 0395/0396/0397/0399/0400/0401/0402/0404/0405/0406. Investigate why Salvo test feature introduces zstd and preserve the tests that depend on that feature.

### SEC-LRU

ssfmt 0.1.2 constrains lru to ^0.16, while the panic-safety fix is in lru >=0.18.2. Check for a compatible ssfmt upstream release first. If none exists, maintain a minimal, documented ssfmt fork/patch or evaluate replacement only after Java formatting parity tests. Do not edit Cargo.lock to violate semver.

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
