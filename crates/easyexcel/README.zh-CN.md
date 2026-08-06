# easyexcel

[English](README.md)

EasyExcel-Rust 面向用户的统一门面，提供 Java EasyExcel 风格的 builder、listener、converter 与 handler。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 编排类型化与动态 XLSX/XLS/CSV 读写。
- 通过 `easyexcel::{model, io, csv, xls, xlsx, formula, markdown, tabular}` 重导出引擎 API。

## 架构

```text
application -> easyexcel builders -> format engines -> spreadsheet files
```

主要公共 API：`EasyExcel, EasyExcelFactory, ExcelRow, ExcelReaderBuilder, ExcelWriterBuilder`。

## 安装与使用

```toml
[dependencies]
easyexcel = "0.1.1"
```

```rust
use easyexcel::{EasyExcel, ExcelRow};

#[derive(ExcelRow)]
struct User {
    #[excel(name = "Name")]
    name: String,
}

let rows = EasyExcel::read_sync::<User>("users.xlsx").do_read_sync()?;
```

## 兼容性与边界

这是 Rust 应用推荐依赖。高级格式无损边界与未支持能力以仓库兼容性矩阵为准。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
