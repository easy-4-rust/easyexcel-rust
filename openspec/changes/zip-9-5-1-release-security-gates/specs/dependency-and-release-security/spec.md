# Dependency and release security specification

## ADDED Requirements

### Requirement: ZIP 9.5.1 minimum without losing Excel semantics

The repository MUST declare ZIP >=9.5.1 on the compatible ZIP 9 line and MUST keep deflate and fallible OOXML entry-name handling intact.

#### Scenario: Excel package round-trip

- GIVEN an XLSX package with workbook sheets and shared entries
- WHEN EasyExcel reads or writes the package
- THEN the normal and malformed-entry regression tests MUST pass
- AND the resolved ZIP package MUST be 9.5.1 or a reviewed later patch within ZIP major 9.

### Requirement: Published Actix adapter default dependency boundary

The published `easyexcel-actix` package MUST NOT enable Actix Web's legacy `http2` feature by default: the workspace-local Actix HTTP patch does not propagate to downstream consumers, and upstream actix-http 3.21.0 resolves h2 0.3.27. The workspace MUST preserve the h2 0.4.20 bridge and its HTTP/2 conformance regression.

#### Scenario: External consumer uses the published adapter alone

- GIVEN the normalized `easyexcel-actix` package is extracted outside the EasyExcel workspace
- WHEN an external Cargo consumer resolves its normal dependencies
- THEN Actix Web default features MUST be disabled by the adapter
- AND the adapter alone MUST NOT introduce h2 0.3.x.

#### Scenario: Workspace HTTP/2 conformance remains enabled

- GIVEN the EasyExcel Web conformance test target
- WHEN it enables `actix-web/http2` as a dev-dependency
- THEN the local Actix HTTP bridge MUST resolve h2 0.4.20
- AND the h2c bridge regression MUST pass.

- NOTE: A downstream application that independently enables legacy `actix-web/http2`, or has another dependency that enables it through Cargo feature unification, can still resolve h2 0.3.27 until upstream Actix ships `http2-native`. This requirement applies to the adapter's own default dependency graph, not arbitrary downstream feature selections.

### Requirement: Strict advisory gating

The Security Audit and Release workflows MUST reject vulnerable, unsound or yanked dependencies without adding exception rules.

#### Scenario: Newly published RustSec advisory

- GIVEN a release commit previously passed PR CI
- AND the advisory database has gained a new applicable warning before release
- WHEN the release build-and-test job runs
- THEN cargo audit --deny warnings MUST fail that job
- AND no publish-crates job may run.

### Requirement: Independent release source and license validation

The release build-and-test job MUST execute cargo deny check advisories bans licenses sources before dry-run-publish and publish-crates.

#### Scenario: Dependency source or license regression

- GIVEN the candidate dependency graph violates a license or source policy
- WHEN cargo deny runs
- THEN the release job MUST fail
- AND there MUST be no new cargo deny allow or source bypass rule.

### Requirement: Auditable evidence

The change MUST preserve a historical baseline and a separate post-merge audit note, with source paths, versions, exact commands, test results, advisory database revision, lockfile hash and GitHub CI references.

#### Scenario: Security change has reproducible evidence

- GIVEN a dependency or release-security change is proposed
- WHEN the change is reviewed
- THEN the audit note MUST identify the source locations, exact versions, verification commands, and CI evidence for the candidate commit.
