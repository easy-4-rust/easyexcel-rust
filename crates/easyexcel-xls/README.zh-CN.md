# easyexcel-xls

[English](README.md)

BIFF8/OLE2 `.xls` 工作簿读取与写入引擎。

> 版本: 0.1.2 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 是 EasyExcel-Rust Workspace 的正式发布模块。本文面向需要理解模块职责、直接调用底层 API 或维护格式引擎的 Rust 开发者。普通业务项目应优先通过 `easyexcel` 门面访问重导出的能力。

## 一览

```text
输入 / 公共 API -> easyexcel-xls -> 类型化模型、行流、文件或报告
```

## 架构

```mermaid
flowchart LR
    File[".xls 文件"] --> CFB["OLE2 / CFB"]
    CFB --> BIFF["BIFF8 记录"]
    BIFF --> Model["easyexcel-model"]
    Model --> Writer["BIFF8 写入器"]
    Writer --> Output[".xls 文件"]
```

依赖方向必须保持从门面或格式引擎指向基础模块；本 crate 不反向依赖业务应用。

## 能力矩阵

| 能力 | 状态 | 说明 |
|:---|:---|:---|
| 工作簿读写 | 可用 | 复合文档识别与 BIFF8 模型映射。 |
| 公式 token | 有边界可用 | 把已支持 BIFF 公式 token 映射到共享模型/引擎。 |
| 事件模式与旧式加密 | 不支持 | 不宣称 XLS Event Mode、旧密码保护或占位符填充。 |

## 公共 API

| API | 用途 |
|:---|:---|
| `read`、`read_path` | 把 XLS 解析为 `Workbook`。 |
| `write`、`write_path` | 把 `Workbook` 编码为 XLS。 |
| `looks_like_cfb`、`CFB_MAGIC` | 容器识别。 |
| `biff8` | 供引擎实现者使用的底层 BIFF8 组件。 |

API 的权威定义来自当前 `src/lib.rs` 重导出与对应实现；README 不把内部私有对象描述为稳定契约。

## 安装

```toml
[dependencies]
easyexcel-xls = "0.1.2"
```

如果项目同时使用多个 EasyExcel 引擎，请改为只依赖 `easyexcel = "0.1.2"`，并通过 `easyexcel::...` 使用，以避免版本漂移。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use easyexcel_xls::{read_path, write_path};

let workbook = read_path(Path::new("input.xls"))?;
write_path(&workbook, Path::new("copy.xls"))?;
Ok(())
}
```

## 进阶使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use easyexcel_model::Cell;
use easyexcel_xls::{read_path, write_path};

let mut workbook = read_path(Path::new("input.xls"))?;
workbook.sheets[0].set_a1("B2", Cell::Text("updated".to_owned()));
write_path(&workbook, Path::new("updated.xls"))?;
Ok(())
}
```

## 错误与能力边界

- XLS 当前使用 Workbook Mode；上层请求 Event Mode 时必须返回类型化不支持错误。
- 业务代码通常应使用 `easyexcel::xls` 或 `EasyExcel` 门面，避免耦合 BIFF 内部实现。

资源限制、格式损失或未支持能力必须通过类型化错误、`Option`、warning 或转换报告显式呈现，禁止静默猜测或降级。

## 依赖关系

```mermaid
flowchart LR
    User["业务代码"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-xls"]
    This --> Foundation["共享基础 crate"]
```

该图表达公共依赖方向，不表示本 crate 必然依赖所有基础模块。实际依赖以 `Cargo.toml` 为准。

## 证据索引

| 声明 | 事实来源 |
|:---|:---|
| 包版本、MSRV 与依赖 | [`Cargo.toml`](Cargo.toml) |
| 公共重导出 | [`src/lib.rs`](src/lib.rs) |
| 实现行为 | `src/xls/ and src/biff8/` |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-xls)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)
