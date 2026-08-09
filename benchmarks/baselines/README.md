# Stable performance baselines

Nightly and release gates consume reviewed reports from this directory. A
baseline is created only from a successful, low-variance run on the same CI
runner class; it must never be synthesized or copied from the historical
numbers in `docs/benchmarks.md`.

Expected names are `nightly-ubuntu-x64.json` and `release-ubuntu-x64.json`.
Until the corresponding file exists, the workflow deliberately reports a
missing-baseline failure while still uploading the candidate result artifact.

The comparator accepts a baseline only from this directory. It must be a
passing report for the same profile and benchmark-spec SHA, contain no gate
failures, and retain non-empty summaries. Release runner attestation includes
the clean Rust source fingerprint, so the reviewed baseline bytes are bound to
the candidate's repository SHA instead of being supplied as an arbitrary JSON
file at comparison time.
