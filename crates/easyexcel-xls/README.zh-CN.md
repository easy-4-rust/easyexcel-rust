# easyexcel-xls

[English](README.md)

BIFF8/OLE2 `.xls` 工作簿读取与写入引擎。

> 版本: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 独立发布是为了支撑 EasyExcel-Rust 内部依赖图。README 面向贡献者和引擎实现者说明模块边界；业务应用应只依赖 `easyexcel`，并使用对应的 `easyexcel::...` 门面路径。

## 一览

```text
业务应用 -> easyexcel:: 门面 -> easyexcel-xls 内部引擎 -> 类型化结果
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
| 公式 token | 有边界可用 | 通过 BIFF8 `SUPBOOK`/`EXTERNSHEET` 支持工作簿内部 `Ref3d`/`Area3d`；外部工作簿引用不在契约内。 |
| 密码加密 | 已编码，待发布证据 | BIFF8 CryptoAPI RC4 使用 `FILEPASS` 与逐记录加解密；非 CryptoAPI 的旧加密方案显式报错。 |
| 占位符填充 | 已编码，待发布证据 | 标量/集合、纵向/横向、重复 fill、`forceNewRow`、样式与关联记录迁移均由 BIFF8 模板引擎承载。 |
| 共享模型适配 | 已编码，待发布证据 | 复用完整 BIFF8 引擎，保留 active Sheet、默认/显式行列尺寸、隐藏状态、行列 XF、小数字号及完整下划线类型。 |
| 事件模式 | 不支持 | XLS 继续使用 Workbook Mode；这与密码和模板能力无关。 |

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
easyexcel = "0.1.3"
```

`easyexcel-xls` 是内部 BIFF8 引擎。业务应用应使用 `easyexcel::xls` 或更高层的 `EasyExcel` builder。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use easyexcel::xls::{read_path, write_path};

let workbook = read_path(Path::new("input.xls"))?;
write_path(&workbook, Path::new("copy.xls"))?;
Ok(())
}
```

## 进阶使用

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
