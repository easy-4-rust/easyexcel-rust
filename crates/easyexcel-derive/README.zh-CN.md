# easyexcel-derive

[English](README.md)

用于 EasyExcel 类型化行映射与注解元数据的过程宏。

> 版本线：0.1.1 · Rust 1.88+ · Edition 2024 · Apache-2.0

## 职责

- 派生 `ExcelRow` schema 与双向行转换。
- 把已支持的 Java EasyExcel 注解语义映射为 `#[excel(...)]` 属性。

## 架构

```text
Rust struct + attributes -> easyexcel-derive -> ExcelRow implementation
```

主要公共 API：`#[derive(ExcelRow)] and #[excel(...)]`。

## 安装与使用

```toml
[dependencies]
easyexcel-derive = "0.1.1"
```

```rust
use easyexcel::ExcelRow;

#[derive(ExcelRow)]
struct OrderRow {
    #[excel(name = "Order ID", index = 0)]
    id: String,
}
```

## 兼容性与边界

用户不应直接依赖此过程宏 crate；`easyexcel` 已重导出 `ExcelRow`。格式渲染边界仍由具体后端决定。

权威能力边界维护在[工作区兼容性矩阵](../../docs/compatibility.md)中。未支持行为必须返回明确错误或警告，禁止静默降级。

## Java 注解映射

| Java 注解 | Rust 属性 |
|---|---|
| `ExcelIgnore` | `ignore` |
| `ExcelIgnoreUnannotated` | `ignore_unannotated` |
| `ExcelProperty` | `property`、`value/head`、`name`、`index`、`order`、`converter` |
| `DateTimeFormat` | `date_time_format`、`use_1904_windowing` |
| `NumberFormat` | `number_format`、`rounding_mode` |
| `ColumnWidth` | `column_width` |
| `ContentFontStyle` | `content_font_style(...)` |
| `ContentLoopMerge` | `content_loop_merge(...)` |
| `ContentRowHeight` | `content_row_height` |
| `ContentStyle` | `content_style(...)` |
| `HeadFontStyle` | `head_font_style(...)` |
| `HeadRowHeight` | `head_row_height` |
| `HeadStyle` | `head_style(...)` |
| `OnceAbsoluteMerge` | `once_absolute_merge(...)` |

`value = ["一级", "二级"]` 对应多级 `ExcelProperty.value()` 表头。启用 `ignore_unannotated` 后，仅配置格式或样式不会让字段进入映射。`default = expression`、`image`、`comment`、`hyperlink`、`formula`、`data_validation`、`conditional` 与 `filter` 是明确标注的 Rust 扩展，不冒充 Java 注解成员。

## 项目链接

- [EasyExcel-Rust](https://github.com/easy-4-rust/easyexcel-rust)
- [API 文档](https://docs.rs/easyexcel-derive)
- [变更日志](../../CHANGELOG.md)
- [英文 README](README.md)
