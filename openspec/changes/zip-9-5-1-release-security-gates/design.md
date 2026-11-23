# Design: ZIP 9.5.1 / release-security gates

## Decision Z1: keep the public implementation stable

Use the existing workspace zip dependency, changing only the minimum compatible version from 9.0.0 to 9.5.1. Preserve default-features=false + deflate. ZIP 9 remains the OOXML package transport; easyexcel-xlsx must continue to propagate fallible entry-name and decoding errors. Cargo.lock is updated using cargo update -p zip --precise 9.5.1, not manual edits.

## Decision Z2: strict advisories are a release invariant

The Security Audit workflow MUST run cargo audit --deny warnings. The release workflow build-and-test MUST independently run both cargo audit --deny warnings and cargo deny check advisories bans licenses sources, and the publish chain MUST depend on the build-and-test job. This is intentional even if a newly published advisory blocks an otherwise green release.

The existing Clippy policy is unchanged (existing lint warnings remain nonfatal under the separately documented CI policy). No audit/deny suppression is added.

## Decision Z3: change isolation and test boundaries

All edits happen in an isolated worktree based on merged main 3ad73e9aa0eb899f85474c9026a4c5f43e8f467e. The user's original local main checkout has unusual staged index removals and untracked paths; do not reset/clean/pull there. Use a dedicated branch. Local builds can use F: CARGO_TARGET_DIR while keeping the source and lockfile under the Git worktree.

Follow-up GitHub PR CI must validate Linux/macOS tests, Rust 1.95 MSRV, Clippy, fmt, docs, coverage-pr, benchmark build and Security Audit. There is no automatic version bump or release.

## Decision Z4: published Actix adapter must not introduce legacy h2 by default

The workspace-local `[patch.crates-io] actix-http` override is not inherited by downstream crates.io consumers. The shared `actix-web` dependency therefore disables default features and explicitly enables the current compression, macros, cookies, Unicode, compatibility and WebSocket features, but not legacy `http2`. The published `easyexcel-actix` package consequently does not implicitly resolve upstream h2 0.3.27.

The workspace HTTP/2 path is preserved: Web conformance explicitly enables `actix-web/http2` as a dev-dependency, and the local Actix HTTP bridge resolves h2 0.4.20. An external application that independently enables Actix's legacy `http2` (or enables it through Cargo feature unification) may still resolve h2 0.3.27. This limitation remains until Actix publishes `http2-native`; do not claim a global downstream HTTP/2 fix.

## Evidence authority

The exact audited source locations, versions, risks, SHA and execution results are in docs/audit/2026-11-15-postmerge-zip-upgrade.md. This design sets the shared boundary; tasks MUST reference that evidence instead of inventing alternate architecture decisions.
