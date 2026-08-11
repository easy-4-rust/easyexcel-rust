# easyexcel-formula

[English](README.md)

> **文档说明**：面向贡献者和引擎实现者说明离线 Excel 公式解析、求值与重算引擎。业务应用应依赖 `easyexcel` 门面。
>
> **版本**：0.1.3
> **最后更新**：2026-08-11

离线 Excel 公式解析、求值、依赖图与重算引擎。

> 版本: 0.1.3 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 概述

本 crate 独立发布是为了支撑 EasyExcel-Rust 内部依赖图。README 面向贡献者和引擎实现者说明模块边界；业务应用应只依赖 `easyexcel`，并使用对应的 `easyexcel::...` 门面路径。

## 一览

```text
业务应用 -> easyexcel:: 门面 -> easyexcel-formula 内部引擎 -> 类型化结果
```

## 架构

```mermaid
flowchart LR
    Text["公式文本"] --> Parser["解析器 / AST 缓存"]
    Parser --> Graph["依赖图"]
    Workbook["Workbook"] --> Evaluator["求值器 + 函数注册表"]
    Graph --> Evaluator
    Evaluator --> Cache["缓存值 + RecalcReport"]
```

依赖方向必须保持从门面或格式引擎指向基础模块；本 crate 不反向依赖业务应用。

## 能力与边界

| 领域 | 能做什么 | 不能做什么 |
|:---|:---|:---|
| 公式解析 | 将公式文本解析为 AST，包含单元格引用、范围引用、函数和表达式。 | 解析 VBA 宏或带外部引用的命名公式。 |
| 单条公式求值 | 通过 `Engine::eval_formula` 在工作簿上下文中求值单条公式。 | 跨多个工作簿求值公式。 |
| 工作簿重算 | 按依赖顺序重算所有公式单元格，更新缓存值并报告循环引用。 | 保证对每种边界情况与 Excel 结果完全一致。 |
| 支持函数 | 算术、统计、文本、逻辑、查找、日期/时间、数学和信息函数。 | Cube、Web、RTD、Pivot 宿主及服务函数（返回明确错误）。 |
| 依赖图 | 构建和遍历公式依赖图以确定重算顺序。 | 跨会话持久化依赖图。 |
| 动态数组 | 对兼容函数支持动态数组溢出。 | 保证与 Excel 365 完全一致的溢出行为。 |
| 错误类型 | 返回带有 Excel 兼容错误码的 `Value::Error`（`#REF!`、`#VALUE!`、`#DIV/0!` 等）。 | 将错误映射为应用特定的错误类型。 |

## 能力矩阵

| 能力 | 状态 | 说明 |
|:---|:---|:---|
| 解析与 AST | 可用 | 单元格/范围引用、函数与表达式。 |
| 工作簿重算 | 可用 | 依赖排序、缓存值更新与循环引用报告。 |
| 外部数据函数 | 不支持 | Cube、Web、RTD、Pivot 宿主及服务函数返回明确错误。 |

## 公共 API

| API | 用途 |
|:---|:---|
| `parse`、`parse_detailed` | 公式文本转 AST。 |
| `Engine::eval_formula` | 在工作簿上下文中计算单条公式。 |
| `Engine::recalc` | 重算公式单元格并更新缓存。 |
| `Value`、`Array`、`CellRef` | 求值结果与引用类型。 |
| `Expr` | AST 表达式节点。 |
| `Context` | 带工作簿引用的求值上下文。 |
| `RecalcReport` | 重算结果，包含求值计数和错误。 |
| `RefRange` | 公式中的单元格范围引用。 |

API 的权威定义来自当前 `src/lib.rs` 重导出与对应实现；README 不把内部私有对象描述为稳定契约。

## 安装

```toml
[dependencies]
easyexcel = "0.1.3"
```

`easyexcel-formula` 是内部公式计算引擎。业务应用应组合使用 `easyexcel::formula` 与 `easyexcel::model`。

| 项目 | 值 |
|:---|:---|
| MSRV | Rust 1.88 |
| Edition | 2024 |
| Resolver | 3 |
| License | Apache-2.0 |

## 基础使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::formula::{CellRef, Engine, Value};
use easyexcel::model::Workbook;

let workbook = Workbook::new();
let mut engine = Engine::new();
let value = engine.eval_formula(
    &workbook,
    CellRef { sheet: 0, row: 0, col: 0 },
    "=SUM(1,2,3)",
);
assert_eq!(value, Value::Number(6.0));
Ok(())
}
```

## 进阶使用

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::formula::Engine;
use easyexcel::model::{Cell, CellValue, Workbook};

let mut workbook = Workbook::new();
workbook.sheets[0].set(
    0,
    0,
    Cell::Formula {
        expr: "1+2".to_owned(),
        cached: CellValue::Empty,
    },
);
let report = Engine::new().recalc(&mut workbook);
println!("recalculated: {}", report.evaluated);
Ok(())
}
```

## 公式解析示例

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
use easyexcel::formula::parse;

let ast = parse("SUM(A1:A10) + B1 * 2");
assert!(ast.is_ok());
Ok(())
}
```

## 错误与能力边界

- 本引擎离线运行，刻意不计算依赖网络服务、OLAP 连接、实时数据或宿主应用状态的函数。
- 函数覆盖范围是显式的，不得把未支持函数描述为完整 Excel 对等。

资源限制、格式损失或未支持能力必须通过类型化错误、`Option`、warning 或转换报告显式呈现，禁止静默猜测或降级。

## 依赖关系

```mermaid
flowchart LR
    User["业务代码"] --> Facade["easyexcel"]
    Facade --> This["easyexcel-formula"]
    This --> Foundation["共享基础 crate"]
```

该图表达公共依赖方向，不表示本 crate 必然依赖所有基础模块。实际依赖以 `Cargo.toml` 为准。

## 证据索引

| 声明 | 事实来源 |
|:---|:---|
| 包版本、MSRV 与依赖 | [`Cargo.toml`](Cargo.toml) |
| 公共重导出 | [`src/lib.rs`](src/lib.rs) |
| 实现行为 | `src/formula/` |
| 跨格式边界 | [Workspace 兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md) |

## 相关链接

- [项目仓库](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-formula)
- [兼容性矩阵](https://github.com/easy-4-rust/easyexcel-rust/blob/main/docs/compatibility.md)
- [变更日志](https://github.com/easy-4-rust/easyexcel-rust/blob/main/CHANGELOG.md)
- [英文 README](README.md)

---

**文档版本**：V1.0.0
**创建日期**：2026-08-11
**最后更新**：2026-08-11
**文档状态**：待评审
