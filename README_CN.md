# easyexcel-rust

[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache--2.0-green.svg)](LICENSE)

**easyexcel-rust** 是阿里巴巴 [EasyExcel](https://github.com/alibaba/easyexcel) 的 Rust 原生移植版本。
以惯用 Rust 方式提供 Java EasyExcel 编程模型：Builder 模式、类型化行映射、事件监听器、类型转换器、流式读取、常量内存写入、模板填充和写入处理器。

Workspace 同时提供 `easyexcel-model`、`easyexcel-formula`、`easyexcel-io`、
XLS/XLSX/CSV 后端和表格转换。library-first 命令应用层由独立 `xls-cli`
产品仓库自行维护；`easyexcel` 门面已不再依赖完整 `xls` fork。

> [架构](docs/ARCHITECTURE.md) · [xls-cli 整合计划](docs/xls-cli-integration-plan.md) · [能力矩阵](docs/xls-cli-capability-matrix.md)

---

## 快速开始

```toml
[dependencies]
easyexcel = "0.1"
```

### 读取 Excel

```rust
use easyexcel::{EasyExcel, ExcelRow, PageReadListener};

#[derive(Debug, ExcelRow)]
struct User {
    #[excel(name = "姓名", index = 0)]
    name: String,
    #[excel(name = "年龄", index = 1)]
    age: Option<u32>,
}

fn main() -> easyexcel::Result<()> {
    // 事件驱动读取（大数据友好）
    let listener = PageReadListener::new(1000, |rows, _ctx| {
        println!("收到 {} 行", rows.len());
    });
    EasyExcel::read::<User, _>("users.xlsx", listener)
        .sheet("用户表")
        .do_read()?;

    // 同步读取（小数据直接获取）
    let users: Vec<User> = EasyExcel::read_sync::<User>("users.xlsx")
        .sheet("用户表")
        .do_read_sync()?;
    
    Ok(())
}
```

### 写入 Excel

```rust
use easyexcel::{EasyExcel, ExcelRow};

#[derive(Debug, ExcelRow)]
#[excel(column_width = 18)]
struct User {
    #[excel(name = "姓名", column_width = 30)]
    name: String,
    #[excel(name = "年龄")]
    age: u32,
    #[excel(name = "生日", format = "yyyy-MM-dd")]
    birthday: chrono::NaiveDate,
}

fn main() -> easyexcel::Result<()> {
    let users = vec![
        User { name: "张三".into(), age: 28, birthday: chrono::NaiveDate::from_ymd_opt(1996, 5, 20).unwrap() },
        User { name: "李四".into(), age: 32, birthday: chrono::NaiveDate::from_ymd_opt(1992, 3, 15).unwrap() },
    ];

    EasyExcel::write::<User>("users.xlsx")
        .sheet("用户表")
        .do_write(users)?;

    Ok(())
}
```

### 模板填充

```rust
use easyexcel::{EasyExcel, TemplateData};

// 简单填充 {key}
let data = TemplateData::new()
    .with("name", "张三")
    .with("date", "2024-01-15");
EasyExcel::fill_template("template.xlsx", "output.xlsx", &data)?;

// 列表填充 {.field}
let list = FillWrapper::new([
    TemplateData::new().with("name", "张三").with("score", 95),
    TemplateData::new().with("name", "李四").with("score", 88),
]);
EasyExcel::fill_template_list("template.xlsx", "output.xlsx", &list, FillConfig::default())?;
```

### 基础组件门面

外部项目只需依赖 `easyexcel`，即可通过统一命名空间使用 CSV、I/O 和工作簿模型：

```rust
use easyexcel::csv::{CsvReadOptions, CsvWriteOptions};
use easyexcel::io::{Format, ResourceLimits};
use easyexcel::model::{Cell, Workbook};
```

这些类型是对应基础 crates 的直接重导出，不是包装类型，也没有额外转换成本。

---

## 核心特性

| 特性 | 支持格式 | 说明 |
|------|---------|------|
| **类型化读写** | XLSX / XLS / CSV | `#[derive(ExcelRow)]` + 注解属性 |
| **事件监听** | XLSX / XLS / CSV | `PageReadListener` / `ReadListener<T>` |
| **流式读取** | XLSX / XLS | SAX 解析，内存可控 |
| **常量内存写入** | XLSX | `SXSSF` 等价实现 |
| **模板填充** | XLSX / XLS | `{key}` / `{.field}` 占位符 |
| **密码加密** | XLSX / XLS | Agile + RC4 |
| **类型转换器** | 全部 | 60+ 内置转换器 |
| **单元格样式** | XLSX / XLS | 字体/填充/对齐/边框 |
| **合并单元格** | XLSX / XLS | `@OnceAbsoluteMerge` / `@ContentLoopMerge` |
| **批注/超链接** | XLSX | 读+写 |
| **图片** | XLSX | 读+写 |
| **公式** | XLSX | 读+写 |
| **CSV BOM** | CSV | 读写支持 |

---

## 注解映射（Java → Rust）

| Java 注解 | Rust 属性 | 说明 |
|-----------|----------|------|
| `@ExcelProperty` | `#[excel(name, index, order, converter)]` | 列映射 |
| `@ExcelIgnore` | `#[excel(ignore)]` | 忽略字段 |
| `@ExcelIgnoreUnannotated` | `#[excel(ignore_unannotated)]` | 忽略未注解 |
| `@DateTimeFormat` | `#[excel(format = "...")]` | 日期格式 |
| `@NumberFormat` | `#[excel(format = "...")]` | 数字格式 |
| `@ColumnWidth` | `#[excel(column_width = N)]` | 列宽 |
| `@HeadRowHeight` | `#[excel(head_row_height = N)]` | 表头行高 |
| `@ContentRowHeight` | `#[excel(content_row_height = N)]` | 内容行高 |
| `@HeadStyle` | `#[excel(head_style(...))]` | 表头样式 |
| `@ContentStyle` | `#[excel(content_style(...))]` | 内容样式 |
| `@HeadFontStyle` | `#[excel(head_font_style(...))]` | 表头字体 |
| `@ContentFontStyle` | `#[excel(content_font_style(...))]` | 内容字体 |
| `@ContentLoopMerge` | `#[excel(content_loop_merge(...))]` | 循环合并 |
| `@OnceAbsoluteMerge` | `#[excel(once_absolute_merge(...))]` | 绝对合并 |

---

## 写入处理器

```rust
use easyexcel::WriteHandler;
use easyexcel_core::{WriteSheetContext, Result, ExcelCellStyle};

struct MyStyleHandler;

impl WriteHandler for MyStyleHandler {
    fn order(&self) -> i32 { 100 }

    fn after_sheet(&mut self, _ctx: &WriteSheetContext) -> Result<()> {
        println!("Sheet written successfully");
        Ok(())
    }

    fn style_cell_style(&self, _ctx: &easyexcel_core::WriteCellContext) -> Option<ExcelCellStyle> {
        // 自定义单元格样式
        None
    }
}

// 注册处理器
EasyExcel::write::<User>("output.xlsx")
    .register_write_handler(MyStyleHandler)
    .sheet("Sheet1")
    .do_write(data)?;
```

---

## 自定义转换器

```rust
use easyexcel_core::{Converter, ReadConverterContext, WriteConverterContext, CellValue, ExcelError};

struct YesNoConverter;

impl Converter<String> for YesNoConverter {
    fn support_excel_type(&self) -> easyexcel_core::CellDataType { easyexcel_core::CellDataType::String }
    
    fn convert_to_rust_data(&self, ctx: &ReadConverterContext) -> Result<String, ExcelError> {
        match ctx.raw_value() {
            CellValue::String(s) if s == "是" => Ok("YES".into()),
            CellValue::String(s) if s == "否" => Ok("NO".into()),
            other => Err(ExcelError::Format(format!("expected 是/否, got {other:?}")))
        }
    }

    fn convert_to_excel_data(&self, ctx: &WriteConverterContext<String>) -> Result<easyexcel_core::WriteCellData, ExcelError> {
        Ok(easyexcel_core::WriteCellData::from_string(
            if ctx.value() == "YES" { "是" } else { "否" }
        ))
    }
}
```

---

## 模块结构

| Crate | 功能 | Java 对应 |
|-------|------|-----------|
| `easyexcel` | 用户入口 Facade | `EasyExcel` / `EasyExcelFactory` |
| `easyexcel-derive` | `#[derive(ExcelRow)]` 过程宏 | `@ExcelProperty` 注解处理 |
| `easyexcel-model` | Workbook、Sheet、Cell 与中立表格模型 | 核心数据模型 |
| `easyexcel-io` | 格式识别、流接口与资源限制 | 读写基础设施 |
| `easyexcel-csv` | CSV 编解码、字符集与流式行源 | CSV 后端 |
| `easyexcel-xls` | BIFF8/OLE2 读写与公式 token | XLS 后端 |
| `easyexcel-xlsx` | OOXML 读写、事件流、模板包与加密 | XLSX 后端 |
| `easyexcel-formula` | 公式 AST、解析、计算与重算 | 公式引擎 |
| `easyexcel-markdown` | GFM 解析、流式输出、策略与损失报告 | Markdown 语义投影 |
| `easyexcel-tabular` | 静态 HTML、JSON 与通用文本格式分派 | 表格交换 |
| `easyexcel-web` | 统一流式 Web 导入导出、限制与错误协议 | Web 执行内核 |

普通用户仍只依赖 `easyexcel`，不直接依赖内部引擎 crate：

```rust
use easyexcel::markdown::{MarkdownConversionMode, MarkdownFormulaPolicy};
use easyexcel::EasyExcel;

let report = EasyExcel::export_markdown("report.xlsx", "report.md")
    .mode(MarkdownConversionMode::Auto)
    .formula_policy(MarkdownFormulaPolicy::CachedValue)
    .do_export()?;

EasyExcel::import_markdown("report.md", "report.xlsx")
    .conservative_types()
    .do_import()?;
```

Markdown 是带结构化损失报告的语义投影，不承诺与 Excel 无损 roundtrip。XLS
使用 Workbook Mode；XLSX 和 CSV 同时支持真实 Event Mode。

---

## Java 兼容性

`easyexcel-rust` 与 Java EasyExcel 4.0.3 保持 1:1 对应：

- **335 个 Java @Test 方法** 全部有 Rust `#[test]` 对应
- **88 个 Golden 测试** 输出与 Java 完全一致
- **152 个 Parity 测试** 端到端行为等价
- 全量测试 **0 FAILEDs**

详见 [迁移文档](docs/migration/TEST_AUDIT_REPORT.md)。

---

## 许可证

Apache-2.0
