# easyexcel-csv

[English](README.md)

支持字符集、分隔符检测、类型推断与增量行流的 CSV/TSV 编解码器。

> 版本: 0.1.2 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 是 EasyExcel-Rust Workspace 的正式发布模块。本文面向需要理解模块职责、直接调用底层 API 或维护格式引擎的 Rust 开发者。普通业务项目应优先通过 `easyexcel` 门面访问重导出的能力。

## 一览

```text
输入 / 公共 API -> easyexcel-csv -> 类型化模型、行流、文件或报告
```

## 架构

```mermaid
flowchart LR
    Input["CSV / TSV 字节"] --> Decode["字符集解码"]
    Decode --> Dialect["分隔符检测"]
    Dialect --> Infer["单元格推断"]
    Infer --> Workbook["工作簿模式"]
    Infer --> Stream["CsvRowSource"]
    Workbook --> Encode["CSV 写入器"]
```

依赖方向必须保持从门面或格式引擎指向基础模块；本 crate 不反向依赖业务应用。

## 能力矩阵

| 能力 | 状态 | 说明 |
|:---|:---|:---|
| 工作簿编解码 | 可用 | 读写一个分隔文本工作表。 |
| 流式行源 | 可用 | 增量 `CsvRowSource`，不全文件 `read_to_end`。 |
| 电子表格专有能力 | 无法表示 | CSV 原生没有样式、公式、合并和多工作表语义。 |

## 公共 API

| API | 用途 |
|:---|:---|
| `CsvReadOptions`、`CsvWriteOptions` | 分隔符、推断与换行策略。 |
| `read_csv`、`write_csv` | 工作簿模式编解码。 |
| `CsvRowSource` | 单次增量行源。 |
| `CsvCharset` | Java 风格字符集名称。 |

API 的权威定义来自当前 `src/lib.rs` 重导出与对应实现；README 不把内部私有对象描述为稳定契约。

## 安装

```toml
[dependencies]
easyexcel-csv = "0.1.2"
```

如果项目同时使用多个 EasyExcel 引擎，请改为只依赖 `easyexcel = "0.1.2"`，并通过 `easyexcel::...` 使用，以避免版本漂移。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel_csv::{CsvReadOptions, CsvWriteOptions, read_csv, write_csv};

let input = "id,name\n1,Alice\n2,Bob\n";
let workbook = read_csv(input.as_bytes(), &CsvReadOptions::default())?;

let mut output = Vec::new();
write_csv(
    &workbook,
    0,
    &mut output,
    &CsvWriteOptions::default(),
)?;
assert!(String::from_utf8(output)?.contains("Alice"));
Ok(())
}
```

## 进阶使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel_csv::{CsvCharset, CsvReadOptions, CsvRowSource};

let options = CsvReadOptions {
    delimiter: Some(b';'),
    infer_types: false,
    sheet_name: "Imported".to_owned(),
};
let source = CsvRowSource::new(
    "code;phone\n007;01012345678\n".as_bytes(),
    options,
    CsvCharset::utf8(),
);
// Call RowSource::stream with an easyexcel_io::RowSink implementation.
Ok(())
}
```

## 错误与能力边界

- 工作簿模式 CSV 每次映射一个工作表；导出多工作表工作簿时调用方必须选择工作表。
- 需要保留前导零标识符等文本时，可以关闭类型推断。

资源限制、格式损失或未支持能力必须通过类型化错误、`Option`、warning 或转换报告显式呈现，禁止静默猜测或降级。

## 依赖关系

```mermaid
flowchart LR
    User["业务代码"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-csv"]
    This --> Foundation["共享基础 crate"]
```

该图表达公共依赖方向，不表示本 crate 必然依赖所有基础模块。实际依赖以 `Cargo.toml` 为准。

## 证据索引

| 声明 | 事实来源 |
|:---|:---|
| 包版本、MSRV 与依赖 | [`Cargo.toml`](Cargo.toml) |
| 公共重导出 | [`src/lib.rs`](src/lib.rs) |
| 实现行为 | `src/csv/` |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-csv)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)
