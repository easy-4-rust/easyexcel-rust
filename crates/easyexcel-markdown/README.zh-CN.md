# easyexcel-markdown

[English](README.md)

面向工作簿与流式行的策略化 Markdown 投影引擎。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 把 GFM 表格解析为 `TabularDocument`。
- 按公式、合并、类型推断和损失报告策略导出工作簿或行流。

## 架构

```text
Workbook / RowSource <-> easyexcel-markdown <-> GFM tables + report
```

主要公共 API：`MarkdownExportOptions, MarkdownImportOptions, MarkdownWriter, MarkdownWorkbookWriter`。

## 安装与使用

```toml
[dependencies]
easyexcel-markdown = "0.1.1"
```

```rust
use easyexcel_markdown::{MarkdownExportOptions, MarkdownProfile, MarkdownWriter};
```

## 兼容性与边界

Markdown 是语义投影而非 Excel 无损往返格式；业务代码应使用 `easyexcel::markdown`。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-markdown)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
