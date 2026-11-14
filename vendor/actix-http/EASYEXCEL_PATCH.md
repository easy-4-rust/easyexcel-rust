# EasyExcel local compatibility patch: Actix HTTP 3.21.0

Upstream package: actix-http 3.21.0
Upstream repository: https://github.com/actix/actix-web
Upstream release: https://github.com/actix/actix-web/releases/tag/actix-http-v3.21.0
Upstream source snapshot: crates.io registry package actix-http 3.21.0

## Why this fork exists

The upstream actix-http 3.21.0 package still declares h2 = 0.3.27, while
the workspace requires a single supported HTTP/2 implementation at h2 0.4.20.
The h2 0.4 line uses http 1.x; Actix HTTP 3.x public request/response types
remain on http 0.2.

## Local changes

1. Change the optional h2 dependency from 0.3.27 to 0.4.20.
2. Add the http1 dependency alias for http 1.5.
3. Bridge HTTP/2 request parts from http 1.x into Actix's http 0.2 request
   head (method, URI, version and repeated headers).
4. Bridge Actix response head from http 0.2 into http 1.x before sending
   through h2 0.4.20.
5. Remove stale upstream #[expect(deprecated)] attributes that are
   unfulfilled by the workspace toolchain.

## Compatibility boundary

- Actix Web public API remains 4.x.
- Actix HTTP public request/response API remains on http 0.2.
- HTTP/2 remains enabled and uses h2 0.4.20.
- HTTP/1, compression, TLS and WebSocket features are retained.
- The bridge is internal to the vendored Actix HTTP implementation.

## Required verification

- Actix HTTP rustls HTTP/2 integration tests.
- EasyExcel Actix adapter and shared conformance tests.
- Chunked request smuggling, Brotli window and decoded-size security regressions.
- cargo tree -i h2 contains only h2 0.4.x.
- cargo audit and cargo deny pass.
