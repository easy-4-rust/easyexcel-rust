.PHONY: test
test:
	cargo test
	RUSTFLAGS='$(RUSTFLAGS) --cfg loom' cargo test loom

.PHONY: test-miri
test-miri:
	cargo +nightly miri test

.PHONY: check
check:
	cargo c --all-targets --all-features
	RUSTFLAGS="$(RUSTFLAGS) --cfg loom" cargo c --all-targets --all-features

.PHONY: lint
lint:
	cargo clippy --all-targets --all-features
	cargo doc --all-features --no-deps
	cargo fmt --check
	RUSTFLAGS="$(RUSTFLAGS) --cfg loom" cargo clippy --all-targets --all-features


.PHONY: ci
ci: export RUSTFLAGS=-Dwarnings
ci: export RUSTDOCFLAGS=-Dwarnings
ci: check lint test test-miri
