# easyexcel-format

[English](README.md)

提供 Java 兼容语义的电子表格数字、日期与显示格式算法。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 解析内建与自定义数字格式。
- 以确定性规则格式化十进制、整数、浮点数和日期值。

## 架构

```text
raw cell value + format code -> easyexcel-format -> display text
```

主要公共 API：`ExcelLocale, NumberRoundingMode, builtin_format_code, format_with_code`。

## 安装与使用

```toml
[dependencies]
easyexcel-format = "0.1.1"
```

```rust
use easyexcel_format::{NumberRoundingMode, builtin_format_code, format_with_code};
```

## 兼容性与边界

本 crate 只格式化值，不读取或写入电子表格容器。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-format)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
