# easyexcel-formula

[English](README.md)

Excel 兼容的公式解析、求值、重算与动态数组引擎。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 把公式解析为表达式模型，并在工作簿上下文中求值。
- 报告重算结果，同时明确保留未支持函数边界。

## 架构

```text
formula text + Workbook -> easyexcel-formula -> value / recalculation report
```

主要公共 API：`Engine, Evaluator, Expr, Context, RecalcReport, parse`。

## 安装与使用

```toml
[dependencies]
easyexcel-formula = "0.1.1"
```

```rust
use easyexcel_formula::{Engine, Expr, RecalcReport, parse};
```

## 兼容性与边界

公式引擎不承诺覆盖 Excel 的全部函数或外部数据函数。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-formula)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
