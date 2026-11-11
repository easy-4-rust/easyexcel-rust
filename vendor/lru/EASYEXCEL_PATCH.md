# Vendored lru 0.16.4 security backport

Upstream source: https://github.com/jeromefroe/lru-rs/tree/0.16.4
Security fix: https://github.com/jeromefroe/lru-rs/pull/238
Merged upstream commit: f9a7f00fcf2d33e00adb03758cb350aaaa52cddb
Advisory: RUSTSEC-2026-0253

The upstream 0.18.2 fix is backported without changing the 0.16.4 package
version or its public API, because ssfmt 0.1.2 requires lru ^0.16.
The only production change detaches the intrusive-list node before dropping
the key. A regression test covers a panicking key destructor and subsequent
cache traversal.
