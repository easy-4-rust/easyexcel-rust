# EasyExcel Rust patch: ref-swap 0.1.3 MSRV compatibility

Upstream: https://github.com/Nitrokey/ref-swap, published crate ref-swap 0.1.3.
Advisory: RUSTSEC-2026-0528 (fixed by upstream 0.1.3).

This vendored copy retains the upstream 0.1.3 soundness fixes:
- RefSwap and OptionRefSwap are invariant in their reference lifetime.
- OptionRefSwap::into_inner returns Option and handles the null pointer.

The only local code delta is replacing the two AtomicPtr::try_update calls in
src/lib.rs with AtomicPtr::fetch_update. fetch_update is stable on Rust 1.88
and has the same compare/update retry semantics and ordering parameters.
Public RefSwap::try_update and OptionRefSwap::try_update APIs are unchanged;
only the internal AtomicPtr primitive is switched.

Three regression tests cover RefSwap update semantics, OptionRefSwap update semantics, and `OptionRefSwap::into_inner(None)` null handling.
The package is intentionally vendored at version 0.1.3 so the advisory fix is
retained without requiring the unstable atomic_try_update API.

Validation:
- cargo +1.88 test --manifest-path vendor/ref-swap/Cargo.toml (3 unit tests + 4 doctests: 2 compile-fail, 2 regular)
- cargo +1.89 check -p easyexcel-rocket --all-features
- cargo +1.89 test -p easyexcel-rocket --all-features (8/8)
- cargo +1.89 check -p easyexcel-demo-rocket --all-features
- cargo audit --no-fetch --deny warnings with refreshed RustSec DB
- cargo deny check advisories bans licenses sources

Do not upgrade this patch by blindly copying a future upstream release; first
verify that it does not reintroduce unstable APIs or alter the safety fixes.