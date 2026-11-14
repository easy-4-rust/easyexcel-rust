# Dependency Security Requirements

## Requirement: Workspace dependencies must be free of known RustSec advisories

The repository MUST NOT retain a locked crate version covered by an active RustSec vulnerability advisory.

### Scenario: Vulnerable dependency upgrade

- GIVEN a dependency version is listed as affected by a RustSec advisory
- WHEN the dependency graph is updated
- THEN every direct and transitive path to that affected version MUST be removed
- AND the fix MUST be verified against the advisory's patched range
- AND the regression test for the affected behavior MUST pass

## Requirement: Unsound dependency warnings are release blockers

The repository MUST NOT suppress unsoundness warnings for dependencies in the locked graph.

### Scenario: Unsound crate remediation

- GIVEN cargo audit reports an unsound dependency
- WHEN the remediation is complete
- THEN the affected version MUST no longer appear in Cargo.lock
- AND no advisory ignore/allow rule may be added
- AND the dependency API/feature/MSRV compatibility MUST be validated

## Requirement: Security remediation must preserve product capabilities

The repository MUST preserve existing framework adapters, HTTP/2, supported compression algorithms, ZIP 9 compatibility and workbook format behavior.

### Scenario: Framework dependency upgrade

- GIVEN Actix, Salvo or Rocket dependency versions change
- WHEN the workspace is built and tested
- THEN the corresponding adapter API and supported protocol/compression features MUST remain available
- AND normal request/response integration tests MUST pass

## Requirement: MSRV boundaries must be explicit per package

The first-party workspace/core/adapter/example/test packages MUST declare or inherit the current workspace MSRV of Rust 1.95, approved by decision D7. Vendored third-party packages may retain their upstream MSRV metadata. Any future MSRV change requires a reviewed decision and CI validation.

### Scenario: Workspace MSRV is Rust 1.95

- GIVEN decision D7 raises the workspace MSRV to Rust 1.95
- WHEN a first-party workspace package is built
- THEN the package MUST declare or inherit Rust 1.95
- AND the complete workspace MUST pass `cargo check --workspace --all-targets --all-features` on Rust 1.95

## Requirement: Audit evidence must be reproducible

Every security audit and remediation MUST record the Git SHA, Cargo.lock hash, toolchain, advisory database revision, commands, exit codes and CI evidence.

### Scenario: Final security gate

- GIVEN all remediation tasks are complete
- WHEN the final audit pipeline runs
- THEN cargo audit --no-fetch --deny warnings MUST exit 0 with zero vulnerabilities, unsound warnings and yanked packages
- AND cargo deny check advisories bans licenses sources MUST exit 0
- AND the test, formatting, lint, MSRV and adapter checks MUST pass
