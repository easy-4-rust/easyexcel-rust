# Proposal: post-merge ZIP upgrade and strict release security

## Why

PR #5 resolved Issue #2 and the historical dependency vulnerabilities, but main still resolves ZIP 9.1.3 and its release workflow does not re-run cargo audit/cargo deny before publishing. ZIP 9.5.1 is a compatible upstream improvement, and future RustSec advisories could otherwise be reported as allowed warnings.

## What changes

- Raise the ZIP dependency floor to 9.5.1, preserving the existing OOXML ZIP 9 API adaptation, deflate and error propagation.
- Lock and audit the new transitive dependencies with no hand-edited lockfile or advisory exceptions.
- Make normal Security Audit and the release publication pipeline reject any cargo audit warning.
- Add the strict cargo deny audit to the release workflow before publication.
- Close the already verified Actix HTTP/2 sensitivity audit task with exact GitHub CI evidence.
- Preserve the workspace Actix HTTP/2 bridge on h2 0.4.20, while excluding upstream legacy `http2` from the published `easyexcel-actix` adapter default dependency graph so the adapter alone does not introduce h2 0.3.27. Record the limitation of application-level feature unification until Actix publishes `http2-native`.
- Keep the historical docs/audit report intact; record the new follow-up separately.

## Non-goals

- Do not release, tag, merge or publish crates in this change without completed acceptance.
- Do not alter Rust 1.95 MSRV, the first-party version 0.1.4, public APIs or framework adapter selection.
- Do not enable additional ZIP encryption/streaming features merely because 9.5.1 offers them.
- Do not add cargo-audit ignore, cargo-deny allow/source exceptions, remove compression, remove the workspace HTTP/2 bridge, or remove the h2 0.4.20 regression test. Do not silently enable legacy Actix `http2` in the published adapter default graph.

## Success

ZIP 9.5.1 resolves and passes XLSX tests; strict local and GitHub security gates pass; the release workflow has an explicit pre-publish security dependency; all required CI checks complete successfully.
