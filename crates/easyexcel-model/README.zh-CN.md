# easyexcel-model

[English](README.md)

EasyExcel-Rust 各格式引擎共享的格式中立工作簿与表格数据模型。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 建模工作簿、工作表、单元格、样式、合并区域、名称、表格和未知部件。
- 在 `Workbook` 与 `TabularDocument` 间转换，但不宣称公式或样式无损往返。

## 架构

```text
XLS / XLSX / CSV engines -> easyexcel-model -> facade / converters
```

主要公共 API：`Workbook, Sheet, Cell, CellValue, CellRange, TabularDocument`。

## 安装与使用

```toml
[dependencies]
easyexcel-model = "0.1.1"
```

```rust
use easyexcel_model::{Cell, CellValue, Workbook};
```

## 兼容性与边界

本 crate 不包含 XLS、XLSX、CSV、ZIP 或 XML 解析器。业务代码通常应通过 `easyexcel::model` 导入这些类型。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-model)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
