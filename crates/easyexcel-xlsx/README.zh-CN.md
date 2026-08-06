# easyexcel-xlsx

[English](README.md)

OOXML `.xlsx` 读取、写入、事件流、模板、加密与往返修改引擎。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 读写工作簿包，并提供事件式工作表读取器。
- 支持模板物化、加密 OOXML 和面向保留的包处理。

## 架构

```text
ZIP / OOXML bytes <-> easyexcel-xlsx <-> Workbook / event stream
```

主要公共 API：`read_path, write_path, XlsxCellEventReader, OoxmlPackage, TemplateFillData`。

## 安装与使用

```toml
[dependencies]
easyexcel-xlsx = "0.1.1"
```

```rust
use easyexcel_xlsx::{XlsxCellEventReader, read_path, write_path};
```

## 兼容性与边界

在支持范围内保留未知 OOXML 部件，但不保证宏、图表和所有高级对象编辑无损；业务代码优先使用 `easyexcel::xlsx`。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-xlsx)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
