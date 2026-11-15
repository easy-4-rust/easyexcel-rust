# Post-merge ZIP 9.5.1 upgrade and release-security gate

## Frozen baseline and source of truth

- Repository: easy-4-rust/easyexcel-rust.
- Base main: 3ad73e9aa0eb899f85474c9026a4c5f43e8f467e (PR #5 merged 2026-11-14).
- Issue #2: closed, fixed through PR #5. PR #8: closed without merging as a superseded ZIP alternative.
- New branch: chore/zip-9-5-1-postmerge-20261115. Work was isolated from an unrelated dirty local main checkout.
- OpenSpec: openspec/changes/zip-9-5-1-release-security-gates/.
- Historical security work: docs/audit/2026-11-08-dependency-security-audit.md. Do not rewrite its original advisory baseline.
- Baseline main GitHub Actions: CI run 43634850008 and Security Audit run 43634850025 both completed successfully.

## Source locations, change and risks

| ID | Source / dependency path | Change | Risk and validation |
|---|---|---|---|
| ZIP-951 | Cargo.toml:136; Cargo.lock zip package; crates/easyexcel-xlsx/src/xlsx/ (OOXML package reader/writer) | Upgrade the declared minimum ZIP 9.0.0 -> 9.5.1; lock ZIP 9.1.3 -> 9.5.1. Keep default-features=false and deflate. | New ZIP reader/index/entry-name and writer internals could change OOXML behavior. Validate XLSX normal/read/write/template/roundtrip and corrupt ZIP error handling. |
| DEP-TRANSITIVE | Cargo.lock zip dependencies | Resolve new constant_time_eq 0.6.1, zeroize_derive 1.5.0 and related compatible dependency graph. | New transitive versions must be audited; inspect cargo tree -p zip, cargo audit and cargo deny. Do not hand-edit Cargo.lock. |
| SEC-STRICT | .github/workflows/security.yml:32-36 | Run cargo audit --deny warnings instead of the default advisory command. | Default cargo audit could exit 0 while reporting allowed unsoundness warnings. Require zero security warnings and run cargo deny advisories/bans/licenses/sources. |
| RELEASE-GATE | .github/workflows/release.yml build-and-test | Add independent strict cargo audit + cargo deny steps before dry-run-publish and publish-crates. | A new advisory may appear between PR CI and a release tag. Re-evaluate the RustSec database during the release itself; stop publishing on failure. |
| ACTIX-REVIEW | vendor/actix-http/src/h2/dispatcher.rs; .github/workflows/ci.yml; OpenSpec T7.7 | Record the completed sensitive-response-header bridge regression. | Verify HPACK HeaderValue sensitivity is preserved. On main Ubuntu CI job 132569293869, the focused vendor test ran and passed. |

## Downstream Actix dependency-boundary audit

### Confirmed pre-fix behavior

The first external-consumer probe used the `easyexcel-actix` package dependency outside the EasyExcel workspace. It resolved:

`h2 0.3.27 -> actix-http 3.21.0 -> actix-web 4.19.0 -> easyexcel-actix 0.1.4`

This proved the root `[patch.crates-io] actix-http = { path = "vendor/actix-http" }` only protects the EasyExcel workspace; it is not propagated through a published library dependency. The local workspace graph alone was insufficient evidence for downstream consumers.

### Upstream status

- Actix issue #4199 tracks upgrading its h2 0.3 dependency.
- Actix PR #4321 proposes a private h2 0.4 backend and `http2-native`, but was still open and draft at this audit date (head `4537c9f5b407c6f7c0e8037b87b8bebbf6388d80`). It is not a released feature and cannot be enabled by this repository yet.

### Mitigation in this change

- Root `actix-web` workspace dependency now disables default features and explicitly enables the current default feature set except legacy `http2`.
- The published `easyexcel-actix` package manifest therefore no longer implicitly enables `actix-web/http2`.
- `tests/easyexcel-web-conformance` opts into `actix-web/http2` only as a dev-dependency. In the EasyExcel workspace this resolves through the local Actix HTTP bridge to h2 0.4.20, preserving the h2c bridge regression test.
- English and Chinese adapter READMEs state the downstream boundary and warn that an application enabling legacy `actix-web/http2` or another dependency enabling it may still reintroduce h2 0.3.27.

### External package evidence

- `cargo package --allow-dirty --no-verify -p easyexcel-actix` succeeded and generated `easyexcel-actix-0.1.4.crate`.
- The normalized packaged `Cargo.toml` has `[dependencies.actix-web] default-features = false` and features `macros`, `compress-brotli`, `compress-gzip`, `compress-zstd`, `cookies`, `unicode`, `compat`, `ws`; it does not include `http2`.
- An isolated consumer using the generated `.crate` package ran `cargo tree -i h2@0.3.27` and `cargo tree -i h2@0.4.20`; both returned the expected “package ID specification did not match any packages” result. The adapter alone introduces neither HTTP/2 implementation.
- A second isolated-consumer run added its own `actix-web = "4.19.0"` dependency with default features; the graph then explicitly resolved `h2 0.3.27 -> actix-http 3.21.0 -> actix-web 4.19.0`. This verifies the documented limitation rather than hiding it.
- Workspace test graph keeps h2 0.4.20 through the conformance dev-dependency and local Actix HTTP patch; `cargo test --locked -p easyexcel-web-conformance --test actix_h2` passed 1/1.
- `cargo +1.95 check --locked -p easyexcel-actix -p easyexcel-web-conformance --all-features` passed; `cargo test --locked -p easyexcel-actix --all-features` passed 10/10; `cargo fmt --all -- --check` and `git diff --check` passed.
- `cargo tree --locked -i h2` for the EasyExcel workspace shows only h2 0.4.20.

### Remaining limitation

This is a safe default for the adapter, not a global guarantee over arbitrary downstream applications. If an application independently enables the legacy Actix `http2` feature or a different dependency enables it through Cargo feature unification, upstream `actix-http 3.21.0` can still pull h2 0.3.27. Revisit when Actix PR #4321 (or an equivalent upstream release) provides a published native h2 0.4 feature. Do not claim that every Actix HTTP/2 consumer is fixed.

## Upstream upgrade evidence

- ZIP upstream releases: https://github.com/zip-rs/zip2/releases/tag/v9.5.1 and https://github.com/zip-rs/zip2/releases/tag/v9.5.0 .
- ZIP v9.5.1 includes the raw_copy_file ZipCrypto metadata fix (#1288); v9.5.0 adds ExtractOptions root_dir filtering. This repo does not enable aes-crypto, and does not claim those optional paths are exercised by its XLSX integration.
- ZIP 9.5.x upstream documents an MSRV of Rust 1.88; easyexcel-rust already requires Rust 1.95 at the first-party workspace level.
- No HTTP/2 adapter, compression, XLS, XLSX, CSV or ZIP 9 migration logic has been removed.

## Local verification snapshot (2026-11-15)

Evidence is from the isolated branch, not a completed GitHub PR/CI on this new branch.

| Command / check | Result |
|---|---|
| cargo update -p zip --precise 9.5.1 | Passed, exit 0 |
| cargo tree --locked -p zip | Resolves zip 9.5.1 |
| cargo test --locked -p easyexcel-xlsx --all-features | 529 passed, 0 failed |
| cargo test --locked -p easyexcel --test calamine_cfb_security | 3 passed, 0 failed |
| cargo audit --no-fetch --deny warnings | Passed, exit 0; no vulnerability/unsound/yanked warning |
| cargo deny check advisories bans licenses sources | Passed, exit 0; no advisory ignore/allow added |
| cargo +1.95 check --locked -p easyexcel-xlsx -p easyexcel --all-features | Passed, exit 0; verifies declared minimum Rust toolchain |
| cargo test --locked -p easyexcel-test --test round_trip | 17 passed, 0 failed |
| cargo test --locked -p easyexcel-actix --all-features | 10 passed, 0 failed |
| cargo test --locked -p easyexcel-web-conformance --test actix_h2 | 1 passed, 0 failed using workspace vendor bridge / h2 0.4.20 |
| cargo package --allow-dirty --no-verify -p easyexcel-actix | Passed; normalized manifest disables default features and omits legacy `http2` |
| Isolated consumer of generated `easyexcel-actix-0.1.4.crate`: `cargo tree -i h2@0.3.27` and `cargo tree -i h2@0.4.20` | Both packages absent; adapter alone introduces no HTTP/2 dependency |
| cargo fmt --all -- --check | Passed, exit 0 |
| git diff --check | Passed, exit 0 |
| go run github.com/rhysd/actionlint/cmd/actionlint@latest .github/workflows/release.yml .github/workflows/security.yml | Passed, exit 0, actionlint 1.7.12 |
| Cargo.lock SHA-256 | 575FADB3DBADE115C34F6AFF6DC88ABEB528BEA1B3BB22783F1CC9EDFC6248DA |
| RustSec advisory-db SHA | 4ef2df8e806a65e2f1ecc14f56f51249062b23d6 |
| Local toolchain | rustc/cargo 1.99.0; cargo-audit 0.22.2; cargo-deny 0.20.2 |

Targeted commands use F: as CARGO_TARGET_DIR because E: has limited available space. This does not change repository artifacts or CI.

## Release and acceptance boundary

- Release and Security YAML validated using actionlint 1.7.12 (exit 0). The mandatory chain is build-and-test (strict security audit and deny included) -> dry-run-publish -> publish-crates.
- Re-run format and focused XLSX / facade tests, and verify ZIP 9.5.1 remains the only resolved ZIP 9 package.
- Run Ubuntu/macOS CI, Clippy, MSRV (Rust 1.95), docs, benchmark build and pull-request coverage on the proposed PR branch.
- Keep releases/tags untouched. The published crates.io version remains v0.1.4. The new main branch includes a breaking Rust 1.95 MSRV change; the release version must be explicitly reviewed under SemVer before tagging.
- Only mark this upgrade accepted after the new PR check suite is green. Security Audit must fail on both vulnerabilities and unsound warnings.
