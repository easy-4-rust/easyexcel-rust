# Benchmark fixtures

Committed fixtures in this directory are immutable inputs. Every file must be
listed with its SHA-256 in `manifest.json`; target-specific normalized or
generated derivatives belong under `benchmarks/results/`, never beside the
authoritative fixture.

The matrix runner also records hashes for Java- and Rust-produced outputs and
feeds both files to both readers. This cross-read evidence complements, but does
not replace, a committed neutral fixture for release claims.
