# easyexcel-markdown

[English](README.md)

> **文档说明**：easyexcel-markdown 引擎层 crate 文档，面向贡献者与引擎实现者说明模块边界。
>
> **版本**：0.1.3
> **最后更新**：2026-08-11

带结构化损失报告、面向工作簿与行流的策略化 GFM 表格导入导出引擎。

> 版本: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 独立发布是为了支撑 EasyExcel-Rust 内部依赖图。README 面向贡献者和引擎实现者说明模块边界；业务应用应只依赖 `easyexcel`，并使用对应的 `easyexcel::...` 门面路径。

## 一览

```text
业务应用 -> easyexcel:: 门面 -> easyexcel-markdown 内部引擎 -> 类型化结果
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

## 格式支持矩阵

本 crate 处理 GFM（GitHub Flavored Markdown）表格语义投影，不是独立的电子表格格式。

| 维度 | GFM 表格（本 crate） | 状态 |
|:---|:---|:---|
| 读取（GFM 表格） | 多表格、最近标题、保守类型推断 | 稳定 |
| 读取（动态/无模型） | `TabularDocument` 输出 | 稳定 |
| 读取（事件监听） | `MarkdownWriter` 作为 `RowSink` | 稳定 |
| 写入（工作簿转 GFM） | 公式/合并/隐藏工作表/显示值策略 | 稳定 |
| 写入（事件模式） | `MarkdownWriter` `RowSink` 实现 | 稳定 |
| 损失报告 | `MarkdownConversionReport` 含机器可读 `MarkdownWarning` | 稳定 |
| 合并单元格 | 策略：`AnchorWithWarning` / `AnchorOnly` / `Error` | 策略驱动 |
| 公式 | 策略：`ExpressionAndCached` / `CachedOnly` / `Error` | 策略驱动 |
| 样式 | GFM 不可表示 | 不支持 |
| 图片 | GFM 表格不可表示 | 不支持 |
| 批注/备注 | GFM 表格不可表示 | 不支持 |
| 超链接 | 仅单元格文本；GFM 表格无原生超链接 | 不支持 |
| 密码保护 | 不适用 | 不适用 |

## 能力与边界

### 本 crate 能做什么

- 将 GFM 表格导入为 `TabularDocument`，附带结构化 `MarkdownConversionReport`。
- 将工作簿导出为 GFM，可配置公式、合并单元格、隐藏工作表和显示值策略。
- 通过 `MarkdownWriter` 作为 `RowSink` 实现 Event Mode 流式输出。
- 对每个损失或降级发出机器可读的 `MarkdownWarning` 代码。

### 本 crate 不能做什么

- Excel 无损往返：Markdown 是语义投影，不是完整的电子表格格式。
- 从 Markdown 文本创建可执行公式：导入时看似公式的文本仍是文本。
- 保留样式、图片、批注、超链接或自动筛选：这些不是 GFM 表格构造。

## 往返保真

Markdown 是语义投影。往返（Excel 转 GFM 再转 Excel）保留：

- 表格结构（行和列）
- 文本和数值单元格值
- 从最近标题派生的表格名称

以下内容通过 `MarkdownConversionReport` 显式报告为损失：

- 合并单元格（可配置策略：仅锚点、锚点+警告或报错）
- 公式（可配置策略：表达式+缓存值、仅缓存值或报错）
- 隐藏工作表（默认排除）
- 样式、图片、批注、超链接、行列尺寸

所有损失通过 `MarkdownWarning` 代码呈现；不发生静默降级。

## 大文件 / 流式 / 内存

| 模式 | 内存复杂度 | 适用场景 |
|:---|:---|:---|
| 工作簿导出（`write_workbook`） | `O(workbook)` | 小到中等工作簿 |
| 事件模式（`MarkdownWriter` RowSink） | `O(batch)` | 大文件流式导出 |
| 导入（`read_markdown`） | `O(document)` | GFM 文档解析 |

`MarkdownWriter` 实现 `RowSink`，支持逐行增量输出，无需缓冲整个工作簿。

## 格式安全

- GFM 解析使用 `pulldown-cmark` 事件流式解析，不物化完整 DOM。
- Markdown 是纯文本格式，无容器、加密或内嵌二进制；ZIP bomb 和实体展开不适用。
- 通过门面调用时，`easyexcel-io::ResourceLimits` 的资源限制生效。

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
easyexcel = "0.1.3"
```

`easyexcel-markdown` 是内部语义投影引擎。业务应用应统一使用稳定的 `easyexcel::markdown` 门面。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::io::Cursor;
use easyexcel::markdown::{MarkdownImportOptions, read_markdown};

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
use easyexcel::markdown::{
    MarkdownExportOptions, MarkdownFormulaPolicy, MarkdownMergePolicy,
    write_workbook,
};
use easyexcel::model::Workbook;

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
| 格式支持矩阵 | [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-markdown)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)

---

**文档版本**：V1.0.0
**最后更新**：2026-08-11
**文档状态**：已评审
