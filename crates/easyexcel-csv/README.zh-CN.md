# easyexcel-csv

[English](README.md)

CSV/TSV 解码、编码、分隔符检测、类型推断与流式行源。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 按可配置字符集和方言读取、写入分隔文本工作簿。
- 提供 `CsvRowSource` 进行增量行处理。

## 架构

```text
CSV / TSV bytes -> easyexcel-csv -> Workbook or row stream
```

主要公共 API：`CsvReadOptions, CsvWriteOptions, CsvRowSource, CsvRecordReader, CsvRecordWriter`。

## 安装与使用

```toml
[dependencies]
easyexcel-csv = "0.1.1"
```

```rust
use easyexcel_csv::{CsvReadOptions, CsvRowSource, CsvWriteOptions};
```

## 兼容性与边界

CSV 原生不具备公式、合并、样式和多工作表语义；业务代码优先使用 `easyexcel::csv`。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-csv)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
