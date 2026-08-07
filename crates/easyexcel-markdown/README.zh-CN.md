# easyexcel-markdown

[English](README.md)

带结构化损失报告、面向工作簿与行流的策略化 GFM 表格导入导出引擎。

> 版本: 0.1.2 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 是 EasyExcel-Rust Workspace 的正式发布模块。本文面向需要理解模块职责、直接调用底层 API 或维护格式引擎的 Rust 开发者。普通业务项目应优先通过 `easyexcel` 门面访问重导出的能力。

## 一览

```text
输入 / 公共 API -> easyexcel-markdown -> 类型化模型、行流、文件或报告
```

## 架构

```mermaid
flowchart LR
    Markdown["GFM 表格"] --> Parser["pulldown-cmark 状态机"]
    Parser --> Document["TabularDocument"]
    Workbook["Workbook"] --> Policy["公式 / 合并 / 值策略"]
    Policy --> Writer["工作簿或 RowSink 写入器"]
    Writer --> Output["UTF-8 GFM + 报告"]
```

依赖方向必须保持从门面或格式引擎指向基础模块；本 crate 不反向依赖业务应用。

## 能力矩阵

| 能力 | 状态 | 说明 |
|:---|:---|:---|
| GFM 导入 | 可用 | 多表格、最近标题与保守类型推断。 |
| 工作簿导出 | 可用 | 公式、合并、隐藏工作表与显示值策略。 |
| Excel 无损往返 | 不宣称 | Markdown 是语义投影。 |

## 公共 API

| API | 用途 |
|:---|:---|
| `MarkdownImportOptions`、`read_markdown` | GFM 转 `TabularDocument` 与报告。 |
| `MarkdownExportOptions`、`write_workbook` | 工作簿转 GFM 与报告。 |
| `MarkdownWriter` | Event Mode 的 `RowSink` 实现。 |
| `MarkdownWarning`、`MarkdownConversionReport` | 机器可读损失信息。 |

API 的权威定义来自当前 `src/lib.rs` 重导出与对应实现；README 不把内部私有对象描述为稳定契约。

## 安装

```toml
[dependencies]
easyexcel-markdown = "0.1.2"
```

如果项目同时使用多个 EasyExcel 引擎，请改为只依赖 `easyexcel = "0.1.2"`，并通过 `easyexcel::...` 使用，以避免版本漂移。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::io::Cursor;
use easyexcel_markdown::{MarkdownImportOptions, read_markdown};

let source = "## Orders\n\n| id | name |\n| --- | --- |\n| 007 | Alice |\n";
let result = read_markdown(
    Cursor::new(source.as_bytes()),
    &MarkdownImportOptions::default(),
)?;
assert_eq!(result.document.tables()[0].name(), "Orders");
Ok(())
}
```

## 进阶使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::io::Cursor;
use easyexcel_markdown::{
    MarkdownExportOptions, MarkdownFormulaPolicy, MarkdownMergePolicy,
    write_workbook,
};
use easyexcel_model::Workbook;

let workbook = Workbook::new();
let options = MarkdownExportOptions::default()
    .with_formulas(MarkdownFormulaPolicy::ExpressionAndCached)
    .with_merges(MarkdownMergePolicy::AnchorWithWarning);
let (output, report) =
    write_workbook(&workbook, Cursor::new(Vec::new()), &options)?;
println!("warnings: {}", report.warnings.len());
Ok(())
}
```

## 错误与能力边界

- 默认 `AgentStable` profile 输出 UTF-8/LF GFM，并明确报告公式/合并损失。
- Markdown 中看似公式的文本导入后仍是文本，导入器不会创建可执行公式。

资源限制、格式损失或未支持能力必须通过类型化错误、`Option`、warning 或转换报告显式呈现，禁止静默猜测或降级。

## 依赖关系

```mermaid
flowchart LR
    User["业务代码"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-markdown"]
    This --> Foundation["共享基础 crate"]
```

该图表达公共依赖方向，不表示本 crate 必然依赖所有基础模块。实际依赖以 `Cargo.toml` 为准。

## 证据索引

| 声明 | 事实来源 |
|:---|:---|
| 包版本、MSRV 与依赖 | [`Cargo.toml`](Cargo.toml) |
| 公共重导出 | [`src/lib.rs`](src/lib.rs) |
| 实现行为 | `src/markdown/` |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-markdown)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)
