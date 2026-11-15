# Dependency and release security specification

## Requirement: ZIP 9.5.1 minimum without losing Excel semantics

The repository MUST declare ZIP >=9.5.1 on the compatible ZIP 9 line and MUST keep deflate and fallible OOXML entry-name handling intact.

### Scenario: Excel package round-trip

- GIVEN an XLSX package with workbook sheets and shared entries
- WHEN EasyExcel reads or writes the package
- THEN the normal and malformed-entry regression tests MUST pass
- AND the resolved ZIP package MUST be 9.5.1 or a reviewed later patch within ZIP major 9.

## Requirement: Strict advisory gating

The Security Audit and Release workflows MUST reject vulnerable, unsound or yanked dependencies without adding exception rules.

### Scenario: Newly published RustSec advisory

- GIVEN a release commit previously passed PR CI
- AND the advisory database has gained a new applicable warning before release
- WHEN the release build-and-test job runs
- THEN cargo audit --deny warnings MUST fail that job
- AND no publish-crates job may run.

## Requirement: Independent release source and license validation

The release build-and-test job MUST execute cargo deny check advisories bans licenses sources before dry-run-publish and publish-crates.

### Scenario: Dependency source or license regression

- GIVEN the candidate dependency graph violates a license or source policy
- WHEN cargo deny runs
- THEN the release job MUST fail
- AND there MUST be no new cargo deny allow or source bypass rule.

## Requirement: Auditable evidence

The change MUST preserve a historical baseline and a separate post-merge audit note, with source paths, versions, exact commands, test results, advisory database revision, lockfile hash and GitHub CI references.
