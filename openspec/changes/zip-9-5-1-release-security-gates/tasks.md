# Tasks: ZIP 9.5.1 post-merge upgrade

Base main: 3ad73e9aa0eb899f85474c9026a4c5f43e8f467e
Branch: chore/zip-9-5-1-postmerge-20261115
Audit source: docs/audit/2026-11-15-postmerge-zip-upgrade.md

## Implement

- [x] Z1.1 Confirm PR #5 merged, PR #8 closed as superseded, Issue #2 closed, and the new worktree is isolated from the dirty local main.
- [x] Z1.2 Verify ZIP upstream 9.5.1 release and documented MSRV; identify Cargo.toml:136 and dependent OOXML source paths.
- [x] Z1.3 Raise ZIP minimum to 9.5.1 and use cargo update to resolve Cargo.lock without removing deflate or adding new ZIP features.
- [x] Z2.1 Change the security workflow to cargo audit --deny warnings.
- [x] Z2.2 Add cargo-audit / cargo-deny installation and checks into the release workflow build-and-test job before dry-run/publish.
- [x] Z2.3 Mark the historical T7.7 sensitive-header review complete based on actual Ubuntu GitHub Actions logs.
- [x] Z3.1 Write this OpenSpec change and the post-merge audit snapshot.

## Validate

- [ ] Z4.9 Revisit Actix HTTP/2 after upstream [PR #4321](https://github.com/actix/actix-web/pull/4321) is merged and a release exposes `http2-native`; do not enable an unpublished feature or silently restore legacy `http2` before then.

- [x] Z4.8 Audit and mitigate the published-consumer Actix dependency boundary. Reproduced the original normalized `easyexcel-actix` package graph resolving upstream `actix-http 3.21.0 -> h2 0.3.27`; confirmed the root workspace patch is not inherited by consumers. The published package now sets `actix-web default-features = false` and enables the current defaults except legacy `http2`. Re-packaged `easyexcel-actix 0.1.4` and tested an isolated external consumer: neither h2 0.3.27 nor h2 0.4.20 is introduced. Workspace conformance dev-dependencies explicitly enable `http2`, using the local vendor bridge to h2 0.4.20; `actix_h2` regression passes 1/1. Limitation, separately reproduced: adding `actix-web = "4.19.0"` with its default features to the external consumer reintroduces h2 0.3.27; another dependency may also enable it through Cargo feature unification. This is documented in both adapter READMEs.

- [x] Z4.1 ZIP version resolved as 9.5.1; cargo audit --no-fetch --deny warnings passes.
- [x] Z4.2 XLSX package unit tests 529/529 and Calamine CFB security tests 3/3 pass locally.
- [x] Z4.3 cargo deny advisories bans licenses sources passes with no new exceptions.
- [x] Z4.4 Rust 1.95 package checks for easyexcel-xlsx/easyexcel pass; public round-trip tests 17/17 pass; cargo fmt and git diff --check pass.
- [x] Z4.5 actionlint 1.7.12 reports zero errors for release.yml and security.yml; release chain is build-and-test (strict audit/deny included) -> dry-run-publish -> publish-crates.
- [ ] Z4.6 Push the isolated branch, open PR and obtain successful Linux/macOS CI, Security Audit and applicable coverage checks.
- [ ] Z4.7 Record the new PR/CI run IDs, final commit SHA and status in the audit doc.

## Not part of this change

- [ ] RELEASE-FOLLOWUP: Review the breaking Rust 1.95 MSRV change and decide the next published SemVer (e.g., 0.2.0 versus maintenance version) before initiating any release. Do not tag or publish in this PR.

## Local evidence

- ZIP 9.1.3 -> 9.5.1; Cargo.lock SHA-256 575FADB3DBADE115C34F6AFF6DC88ABEB528BEA1B3BB22783F1CC9EDFC6248DA.
- rustc/cargo 1.99.0, cargo-audit 0.22.2, cargo-deny 0.20.2, RustSec db 4ef2df8e806a65e2f1ecc14f56f51249062b23d6.
- XLSX 529 passed; Calamine CFB 3 passed; strict audit and deny passed.
- Rust 1.95 package check succeeded; facade round-trip 17/17; fmt/diff-check succeeded; actionlint 1.7.12 passed against the two edited workflow files.
