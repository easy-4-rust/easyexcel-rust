# easyexcel-xls

[English](README.md)

> **文档说明**：easyexcel-xls 引擎层 crate 文档，面向贡献者与引擎实现者说明模块边界。
>
> **版本**：0.1.3
> **最后更新**：2026-08-11

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

## 格式支持矩阵

数据来源：[`docs/ARCHITECTURE.md` File Format Support](../../docs/ARCHITECTURE.md)。

| 维度 | XLS（本 crate） | 状态 |
|:---|:---|:---|
| 读取（类型化行） | 通过 `calamine` + BIFF handler 解析 BIFF8 记录 | 稳定 |
| 读取（动态/无模型） | 支持 | 稳定 |
| 读取（事件监听） | 仅 Workbook Mode；不支持 Event Mode | 不适用 |
| 读取（密码保护） | BIFF8 CryptoAPI RC4，通过 `FILEPASS` | 稳定 |
| 写入（类型化行） | 自定义 BIFF8 编码器 | 稳定 |
| 写入（带密码） | BIFF8 CryptoAPI RC4 | 稳定 |
| 写入（常量内存） | 不支持；XLS 始终全量物化 | 不支持 |
| 模板填充（`{key}` 标量） | 基于 LABEL 的标量替换 | 稳定 |
| 模板填充（列表 `{.}`） | 集合填充，支持纵向/横向/重复 | 稳定 |
| 合并单元格 | 支持 | 稳定 |
| 列宽 | 支持 | 稳定 |
| 行高 | 支持 | 稳定 |
| 样式（字体/填充/对齐） | 基础：FONT/XF/FORMAT/调色板分配 | 稳定 |
| 批注/备注 | 仅读取 | 稳定 |
| 超链接 | 仅读取 | 稳定 |
| 图片 | 仅写入 | 稳定 |
| 公式 | Ref3d/Area3d 通过 SUPBOOK/EXTERNSHEET；外部工作簿引用排除 | 有限 |
| 自动筛选 | 不支持 | 不支持 |

## 能力与边界

### 本 crate 能做什么

- 检测 OLE2/CFB 容器（`looks_like_cfb`、`CFB_MAGIC`），将 BIFF8 记录映射到共享 `Workbook` 模型。
- 通过 `read`/`read_path` 和 `write`/`write_path` 读写 XLS 工作簿。
- 解密和加密 BIFF8 CryptoAPI RC4 保护文件（`read_path_with_password`、`write_path_with_password`）。
- 用标量和集合占位符填充 XLS 模板，包括 `forceNewRow`、样式迁移和关联记录迁移。
- 往返时保留活动工作表、默认/显式行列尺寸、隐藏状态、行列 XF、小数字号和所有 BIFF8 下划线模式。

### 本 crate 不能做什么

- 事件模式读取：XLS 始终使用 Workbook Mode；上层请求 Event Mode 时返回类型化不支持错误。
- 常量内存（SXSSF）写入：XLS 全量物化工作簿；仅 XLSX 支持 `O(window)` 写入。
- 写入批注、超链接或公式：这些是 XLS 的只读能力。
- 外部工作簿公式引用：仅支持通过 `SUPBOOK`/`EXTERNSHEET` 的工作簿内部 `Ref3d`/`Area3d`。
- 非 CryptoAPI 的旧加密方案：显式报错。

## 往返保真

读取后原样写入 XLS 文件时，本 crate 保留：

- 活动工作表选择和工作表排序
- 默认/显式行列尺寸
- 工作表、行和列的隐藏状态
- 行列 XF（格式）记录
- 小数字号和所有 BIFF8 下划线模式

未知或不支持的 BIFF8 记录类型在记录帧允许的范围内保留二进制内容。损失必须通过类型化错误、`Option`、warning 或转换报告显式呈现，禁止静默猜测或降级。

## 大文件 / 流式 / 内存

| 模式 | 内存复杂度 | 适用场景 |
|:---|:---|:---|
| Workbook Mode（默认） | `O(document)` | 所有 XLS 读取；需要全量物化 |
| LazySst（feature `xls-lazy-sst`） | 延迟 SST 解码 | 构造加速 61.8x；首次访问时解码字符串 |
| StreamingRecordIter（feature `xls-streaming-iter`） | 无全量子流 `Vec<u8>` | 从 `BufRead + Seek` 流式读取 BIFF 记录 |

XLS 不支持 XLSX 风格的 `O(window)` 常量内存写入路径。大规模导出请使用 XLSX 格式。

## 格式安全

- OLE2 容器解析使用 `cfb` crate，有界记录帧；BIFF8 记录长度按格式规范固定为 `u8`/`u16`/`u32` 位域。
- BIFF8 CryptoAPI RC4 使用 `md-5` + `getrandom` 加密；非 CryptoAPI 旧方案被显式拒绝。
- XLS 不是基于 ZIP 的格式，因此 ZIP bomb 保护不适用。
- 通过门面调用时，`easyexcel-io::ResourceLimits` 的资源限制生效。

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
| 格式支持矩阵 | [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-xls)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)

---

**文档版本**：V1.0.0
**最后更新**：2026-08-11
**文档状态**：已评审
