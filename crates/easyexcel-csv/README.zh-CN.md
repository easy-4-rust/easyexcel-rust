# easyexcel-csv

[English](README.md)

> **文档说明**：easyexcel-csv 引擎层 crate 文档，面向贡献者与引擎实现者说明模块边界。
>
> **版本**：0.1.3
> **最后更新**：2026-08-11

支持字符集、分隔符检测、类型推断与增量行流的 CSV/TSV 编解码器。

> 版本: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 独立发布是为了支撑 EasyExcel-Rust 内部依赖图。README 面向贡献者和引擎实现者说明模块边界；业务应用应只依赖 `easyexcel`，并使用对应的 `easyexcel::...` 门面路径。

## 一览

```text
业务应用 -> easyexcel:: 门面 -> easyexcel-csv 内部引擎 -> 类型化结果
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

## 格式支持矩阵

数据来源：[`docs/ARCHITECTURE.md` File Format Support](../../docs/ARCHITECTURE.md)。

| 维度 | CSV（本 crate） | 状态 |
|:---|:---|:---|
| 读取（类型化行） | `csv` crate + `encoding_rs` 字符集解码 | 稳定 |
| 读取（动态/无模型） | 支持 | 稳定 |
| 读取（事件监听） | `CsvRowSource` 增量流式 | 稳定 |
| 读取（密码保护） | CSV 格式不适用 | 不适用 |
| 写入（类型化行） | `csv` crate 编码器 | 稳定 |
| 写入（带密码） | CSV 格式不适用 | 不适用 |
| 写入（常量内存） | 通过 `CsvRecordWriter` 行级流式 | 稳定 |
| 模板填充 | CSV 格式不适用 | 不适用 |
| 合并单元格 | 非 CSV 原生语义 | 不支持 |
| 列宽 | 非 CSV 原生语义 | 不支持 |
| 行高 | 非 CSV 原生语义 | 不支持 |
| 样式（字体/填充/对齐） | 非 CSV 原生语义 | 不支持 |
| 批注/备注 | 非 CSV 原生语义 | 不支持 |
| 超链接 | 非 CSV 原生语义 | 不支持 |
| 图片 | 非 CSV 原生语义 | 不支持 |
| 公式 | 非 CSV 原生语义 | 不支持 |
| 自动筛选 | 非 CSV 原生语义 | 不支持 |

## 能力与边界

### 本 crate 能做什么

- 通过 `read_csv`/`write_csv` 读写每个 CSV/TSV 文件的一个分隔文本工作表。
- 通过 `CsvRowSource` 增量流式读取行，无需读取整个文件。
- 检测分隔符、处理 BOM 标记、通过 `CsvCharset`（Java 风格名称）解码多种字符集。
- 推断单元格类型（数值、日期、文本），可通过 `CsvReadOptions.infer_types` 关闭。
- 通过 `CsvWriteOptions` 配置分隔符、换行策略和编码。

### 本 crate 不能做什么

- 多工作表工作簿：CSV 每次映射一个工作表；导出时调用方必须选择工作表。
- 样式、公式、合并单元格、图片、批注、超链接和自动筛选：这些不是 CSV 原生语义。
- 密码保护：CSV 格式不适用。
- 全文件 `read_to_end`：流式 `CsvRowSource` 不缓冲整个文件。

## 往返保真

CSV 是电子表格数据的有损投影。往返（读取后写入）保留：

- 单元格值（文本、数值、日期），保持字符集保真
- 分隔符和换行策略（配置一致时）

以下内容在 CSV 导出时丢失：样式、公式、合并单元格、多工作表、图片、批注、超链接、行列尺寸和自动筛选。这些损失是 CSV 格式的固有特性，不是实现缺陷。

## 大文件 / 流式 / 内存

| 模式 | 内存复杂度 | 适用场景 |
|:---|:---|:---|
| 工作簿模式（`read_csv`） | `O(sheet)` | 小到中等文件 |
| 流式模式（`CsvRowSource`） | `O(batch)` | 大文件批量导入 |
| 写入（`write_csv`/`CsvRecordWriter`） | `O(row)` | 所有写入均为行级流式 |

CSV 天然支持行级流式，无临时文件开销。`CsvRowSource` 增量源避免全文件物化。

## 格式安全

- CSV 是纯文本格式，无容器、加密或内嵌二进制；ZIP bomb 和实体展开不适用。
- 字符集解码使用 `encoding_rs`，有界缓冲分配。
- 分隔符检测读取输入的有界前缀。
- 通过门面调用时，`easyexcel-io::ResourceLimits` 的资源限制生效。

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
easyexcel = "0.1.3"
```

`easyexcel-csv` 独立发布仅用于内部依赖图分层。业务应用应统一使用稳定的 `easyexcel::csv` 门面。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::csv::{CsvReadOptions, CsvWriteOptions, read_csv, write_csv};

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
use easyexcel::csv::{CsvCharset, CsvReadOptions, CsvRowSource};

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
// Call RowSource::stream with an easyexcel::io::RowSink implementation.
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
| 格式支持矩阵 | [ARCHITECTURE.md](../../docs/ARCHITECTURE.md) |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-csv)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)

---

**文档版本**：V1.0.0
**最后更新**：2026-08-11
**文档状态**：已评审
