# easyexcel-derive

[English](README.md)

实现类型化 EasyExcel 行 schema、转换与 Java 注解元数据的过程宏。

> 版本: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 独立发布是为了支撑 EasyExcel-Rust 内部依赖图。README 面向贡献者和引擎实现者说明模块边界；业务应用应只依赖 `easyexcel`，并使用对应的 `easyexcel::...` 门面路径。

## 一览

```text
业务应用 -> easyexcel:: 门面 -> easyexcel-derive 内部引擎 -> 类型化结果
```

## 架构

```mermaid
flowchart LR
    Struct["Rust 结构体"] --> Parser["syn 属性解析器"]
    Parser --> Metadata["注解模型"]
    Metadata --> Expand["quote 代码生成"]
    Expand --> Trait["ExcelRow 实现"]
    Trait --> Facade["easyexcel builders"]
```

依赖方向必须保持从门面或格式引擎指向基础模块；本 crate 不反向依赖业务应用。

## 能力矩阵

| 能力 | 状态 | 说明 |
|:---|:---|:---|
| 类型化行派生 | 可用 | 生成 schema 与双向行转换。 |
| Java 注解语义 | 受后端边界约束可用 | 十四类注解映射为 `#[excel(...)]`。 |
| Rust 扩展 | 可用 | 公式、图片、批注、超链接、校验、条件与过滤元数据。 |

## 公共 API

| API | 用途 |
|:---|:---|
| `#[derive(ExcelRow)]` | 生成行 schema 与转换实现。 |
| `#[excel(name/index/order/...)]` | 列映射元数据。 |
| 样式属性 | 表头/内容字体、样式、宽高与合并元数据。 |
| 格式属性 | 日期时间、数字格式与舍入模式。 |

API 的权威定义来自当前 `src/lib.rs` 重导出与对应实现；README 不把内部私有对象描述为稳定契约。

## 安装

```toml
[dependencies]
easyexcel = "0.1.3"
```

`easyexcel-derive` 是过程宏实现细节。业务应用应导入 `easyexcel::ExcelRow`；直接依赖宏 crate 不属于推荐的公共用法。

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::ExcelRow;

#[derive(Debug, ExcelRow)]
#[excel(column_width = 18, head_row_height = 24)]
struct OrderRow {
    #[excel(value = ["Order", "ID"], index = 0)]
    id: String,

    #[excel(name = "Amount", number_format = "0.00")]
    amount: f64,
}
Ok(())
}
```

## 进阶使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::ExcelRow;

#[derive(ExcelRow)]
#[excel(ignore_unannotated)]
struct StrictRow {
    #[excel(property, name = "Included")]
    included: String,

    // Style-only metadata does not opt this field into strict mapping.
    #[excel(number_format = "0.00")]
    ignored: f64,

    #[excel(ignore, default = String::new())]
    internal: String,
}
Ok(())
}
```

## 注解映射

| Java 注解 | Rust 属性 |
|:---|:---|
| `ExcelIgnore` | `ignore` |
| `ExcelIgnoreUnannotated` | `ignore_unannotated` |
| `ExcelProperty` | `property`、`value/head`、`name`、`index`、`order`、`converter` |
| `DateTimeFormat` | `date_time_format`、`use_1904_windowing` |
| `NumberFormat` | `number_format`、`rounding_mode` |
| `ColumnWidth` | `column_width` |
| `ContentFontStyle` / `HeadFontStyle` | `content_font_style(...)` / `head_font_style(...)` |
| `ContentStyle` / `HeadStyle` | `content_style(...)` / `head_style(...)` |
| `ContentLoopMerge` | `content_loop_merge(...)` |
| `ContentRowHeight` / `HeadRowHeight` | `content_row_height` / `head_row_height` |
| `OnceAbsoluteMerge` | `once_absolute_merge(...)` |

多级 `ExcelProperty.value()` 映射为 `value = ["一级", "二级"]`。`default = expression` 是明确记录的 Rust 扩展。

## 错误与能力边界

- 用户应通过 `easyexcel::ExcelRow` 使用该宏，不应直接把过程宏 crate 作为运行时依赖。
- 元数据支持与文件格式渲染是两个层次，具体后端限制仍是权威边界。

资源限制、格式损失或未支持能力必须通过类型化错误、`Option`、warning 或转换报告显式呈现，禁止静默猜测或降级。

## 依赖关系

```mermaid
flowchart LR
    User["业务代码"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-derive"]
    This --> Foundation["共享基础 crate"]
```

该图表达公共依赖方向，不表示本 crate 必然依赖所有基础模块。实际依赖以 `Cargo.toml` 为准。

## 证据索引

| 声明 | 事实来源 |
|:---|:---|
| 包版本、MSRV 与依赖 | [`Cargo.toml`](Cargo.toml) |
| 公共重导出 | [`src/lib.rs`](src/lib.rs) |
| 实现行为 | `src/annotation/ and src/expand/` |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-derive)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)
