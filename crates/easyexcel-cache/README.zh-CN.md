# easyexcel-cache

[English](README.md)

流式电子表格读取器使用的共享字符串缓存后端。

> 版本: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 独立发布是为了支撑 EasyExcel-Rust 内部依赖图。README 面向贡献者和引擎实现者说明模块边界；业务应用应只依赖 `easyexcel`，并使用对应的 `easyexcel::...` 门面路径。

## 一览

```text
业务应用 -> easyexcel:: 门面 -> easyexcel-cache 内部引擎 -> 类型化结果
```

## 架构

```mermaid
flowchart LR
    SST["sharedStrings.xml"] --> Policy["SharedStringCachePolicy"]
    Policy --> Memory["内存"]
    Policy --> File["临时文件"]
    Policy --> Moka["Moka 对象"]
    Memory --> Reader["索引读取器"]
    File --> Reader
    Moka --> Reader
```

依赖方向必须保持从门面或格式引擎指向基础模块；本 crate 不反向依赖业务应用。

## 能力矩阵

| 能力 | 状态 | 说明 |
|:---|:---|:---|
| 内存缓存 | 可用 | 快速顺序写入与不可变读取视图。 |
| 文件缓存 | 可用 | 大型共享字符串表使用临时文件存储。 |
| Moka 对象缓存 | 可用 | 缓存生命周期内不配置容量、TTL 或 TTI 淘汰。 |

## 公共 API

| API | 用途 |
|:---|:---|
| `SharedStringCachePolicy` | 内存/文件选择阈值。 |
| `ReadCacheMode` | 自动、内存、文件或 Moka 模式。 |
| `SharedStringCacheWriter` | 顺序填充。 |
| `SharedStringCacheReader` | 并发索引读取。 |

API 的权威定义来自当前 `src/lib.rs` 重导出与对应实现；README 不把内部私有对象描述为稳定契约。

## 安装

```toml
[dependencies]
easyexcel = "0.1.3"
```

`easyexcel-cache` 是内部共享字符串缓存引擎。业务应用应通过 `EasyExcel` 读取 builder 配置缓存，不直接构造引擎缓存。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::{EasyExcel, ExcelRow, ReadCacheMode};

#[derive(Debug, ExcelRow)]
struct Row {
    name: String,
}

let rows = EasyExcel::read_sync::<Row>("input.xlsx")
    .read_cache(ReadCacheMode::Memory)
    .do_read_sync()?;
println!("rows: {}", rows.len());
Ok(())
}
```

## 进阶使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::{
    EasyExcel, ExcelRow, SimpleReadCacheSelector, StoredReadCacheSelector,
};

#[derive(Debug, ExcelRow)]
struct Row {
    value: String,
}

let rows = EasyExcel::read_sync::<Row>("large.xlsx")
    .read_cache_selector(StoredReadCacheSelector::Simple(
        SimpleReadCacheSelector::new(),
    ))
    .do_read_sync()?;
println!("rows: {}", rows.len());
Ok(())
}
```

## 错误与能力边界

- 本缓存保存解码后的共享字符串，不缓存任意工作簿或业务对象。
- Moka 后端刻意不在读取过程中淘汰条目，结束时由所有权整体释放。

资源限制、格式损失或未支持能力必须通过类型化错误、`Option`、warning 或转换报告显式呈现，禁止静默猜测或降级。

## 依赖关系

```mermaid
flowchart LR
    User["业务代码"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-cache"]
    This --> Foundation["共享基础 crate"]
```

该图表达公共依赖方向，不表示本 crate 必然依赖所有基础模块。实际依赖以 `Cargo.toml` 为准。

## 证据索引

| 声明 | 事实来源 |
|:---|:---|
| 包版本、MSRV 与依赖 | [`Cargo.toml`](Cargo.toml) |
| 公共重导出 | [`src/lib.rs`](src/lib.rs) |
| 实现行为 | `src/cache/` |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-cache)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)
