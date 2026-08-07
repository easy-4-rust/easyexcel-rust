# easyexcel-xls

[简体中文](README.zh-CN.md)

BIFF8/OLE2 `.xls` workbook reader and writer.

> Release: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## Overview

This crate is independently published to support the EasyExcel-Rust internal dependency graph. Its README documents the engine boundary for contributors and engine implementors; application code should depend on `easyexcel` and use the matching `easyexcel::...` facade path.

## At a glance

```text
Application -> easyexcel:: facade -> easyexcel-xls internal engine -> typed result
```

## Architecture

```mermaid
flowchart LR
    File[".xls file"] --> CFB["OLE2 / CFB"]
    CFB --> BIFF["BIFF8 records"]
    BIFF --> Model["easyexcel-model"]
    Model --> Writer["BIFF8 writer"]
    Writer --> Output[".xls file"]
```

Dependency direction remains from the facade or format engines toward foundations; this crate never depends back on an application.

## Capability matrix

| Capability | Status | Details |
|:---|:---|:---|
| Workbook read/write | Available | Compound File Binary detection and BIFF8 mapping. |
| Formula tokens | Available with boundaries | Maps supported BIFF formula tokens through the shared model/engine. |
| Event mode and legacy encryption | Unsupported | No claimed XLS Event Mode, legacy password protection or placeholder fill. |

## Public API

| API | Purpose |
|:---|:---|
| `read`, `read_path` | Parse XLS into `Workbook`. |
| `write`, `write_path` | Encode `Workbook` as XLS. |
| `looks_like_cfb`, `CFB_MAGIC` | Container recognition. |
| `biff8` | Low-level BIFF8 components for engine implementors. |

The current `src/lib.rs` re-exports and their implementations are authoritative. This README does not present private implementation objects as stable API.

## Installation

```toml
[dependencies]
easyexcel = "0.1.3"
```

`easyexcel-xls` is the internal BIFF8 engine. Applications should use `easyexcel::xls` or the high-level `EasyExcel` builders.

## Basic usage

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use easyexcel::xls::{read_path, write_path};

let workbook = read_path(Path::new("input.xls"))?;
write_path(&workbook, Path::new("copy.xls"))?;
Ok(())
}
```

## Advanced usage

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use easyexcel::model::Cell;
use easyexcel::xls::{read_path, write_path};

let mut workbook = read_path(Path::new("input.xls"))?;
workbook.sheets[0].set_a1("B2", Cell::Text("updated".to_owned()));
write_path(&workbook, Path::new("updated.xls"))?;
Ok(())
}
```

## Errors and capability boundaries

- XLS currently uses Workbook Mode. Requesting Event Mode through higher layers must return a typed unsupported error.
- Application code should normally use `easyexcel::xls` or the `EasyExcel` facade rather than coupling to BIFF internals.

Resource limits, format loss and unsupported behavior must surface through typed errors, `Option`, warnings or conversion reports; silent guessing and downgrade are forbidden.

## Relationship to other crates

```mermaid
flowchart LR
    User["Application"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-xls"]
    This --> Foundation["Shared foundation crates"]
```

The diagram shows the public dependency direction, not that this crate depends on every foundation module. `Cargo.toml` is authoritative.

## Evidence map

| Claim | Source of truth |
|:---|:---|
| Package version, MSRV and dependencies | [`Cargo.toml`](Cargo.toml) |
| Public exports | [`src/lib.rs`](src/lib.rs) |
| Implementation behavior | `src/xls/ and src/biff8/` |
| Cross-format boundaries | [Workspace compatibility matrix](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## Related links

- [Repository](https://github.com/easy-4-rust/easyexcel-rust)
- [API documentation](https://docs.rs/easyexcel-xls)
- [Compatibility matrix](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [Changelog](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [Chinese README](README.zh-CN.md)
