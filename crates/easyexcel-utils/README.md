# easyexcel-utils

[简体中文](README.zh-CN.md)

Reusable Java-compatible string, collection, coordinate and validation algorithms.

> Release line: 0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Responsibilities

- Provides small deterministic helpers used across engines.
- Keeps reusable algorithms out of the public EasyExcel facade orchestration layer.

## Architecture

```text
engine input -> easyexcel-utils helpers -> normalized values
```

Main public surface: `string_utils, coordinate_utils, list_utils, map_utils, validation`.

## Installation and usage

```toml
[dependencies]
easyexcel-utils = "0.1.1"
```

```rust
use easyexcel_utils::{coordinate_utils, string_utils, validation};
```

## Compatibility and limits

This is an internal engine crate, not an alternative facade or general-purpose utility framework.

The authoritative capability boundaries are maintained in the [workspace compatibility matrix](../../docs/compatibility.md). Unsupported behavior must return an explicit error or warning rather than silently downgrade.

## Project links

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-utils)
- [Changelog](../../CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
