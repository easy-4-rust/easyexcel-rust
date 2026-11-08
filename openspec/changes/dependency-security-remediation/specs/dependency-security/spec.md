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

The root/core MSRV MUST remain 1.88 unless a separately reviewed decision changes it. An adapter that requires a higher compiler MUST declare and validate its own MSRV.

### Scenario: Adapter requires Rust 1.89

- GIVEN an upstream framework release requires Rust 1.89
- WHEN the adapter adopts that release
- THEN the adapter/example packages MUST accurately declare Rust 1.89
- AND the core packages MUST still pass their Rust 1.88 checks

## Requirement: Audit evidence must be reproducible

Every security audit and remediation MUST record the Git SHA, Cargo.lock hash, toolchain, advisory database revision, commands, exit codes and CI evidence.

### Scenario: Final security gate

- GIVEN all remediation tasks are complete
- WHEN the final audit pipeline runs
- THEN cargo audit --no-fetch --deny warnings MUST exit 0 with zero vulnerabilities, unsound warnings and yanked packages
- AND cargo deny check advisories bans licenses sources MUST exit 0
- AND the test, formatting, lint, MSRV and adapter checks MUST pass
