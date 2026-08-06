# easyexcel-cache

[简体中文](README.zh-CN.md)

Reusable shared-string caches for streaming spreadsheet readers.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Provides memory, file-backed and Moka-backed shared-string caches.
- Selects cache implementations through a common policy and handle API.

## Architecture

```text
shared strings -> cache policy -> memory / file / Moka cache -> reader
```

Main public surface: `SharedStringCache, SharedStringCachePolicy, ReadCacheMode, create_cache`.

## Installation and usage

```toml
[dependencies]
easyexcel-cache = "0.1.1"
```

```rust
use easyexcel_cache::{ReadCacheMode, SharedStringCachePolicy, create_cache};
```

## Compatibility and limits

This crate caches shared strings; it is not a workbook cache and does not implement eviction-sensitive business semantics.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-cache)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
