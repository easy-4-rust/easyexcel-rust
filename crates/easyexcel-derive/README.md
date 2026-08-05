# easyexcel-derive

`easyexcel-derive` 是 EasyExcel-Rust 的内部过程宏 crate。Rust 用户不直接依赖它，而是始终从门面 crate 使用：

```rust
use easyexcel::{EasyExcel, ExcelRow};

#[derive(ExcelRow)]
struct OrderRow {
    #[excel(value = ["订单", "编号"], index = 0)]
    id: String,
    #[excel(name = "金额", number_format = "0.00", rounding_mode = "HALF_UP")]
    amount: f64,
}
```

过程宏源码按“解析模型 → 注解域解析器 → 代码生成”分层：`annotation/` 只负责
`#[excel(...)]` 语义，`expand/` 只负责生成 `ExcelRow` 实现；共享解析逻辑位于
`annotation/style_parser/`、`expand/conversion/` 等目录的 `mod.rs` 中，`mod.rs`
仅声明模块和重导出符号，不承载对象实现。

## Java 注解对照

| Java 注解 | Rust 辅助属性 | 实现文件 |
|---|---|---|
| `ExcelIgnore` | `ignore` | `annotation/excel_ignore.rs` |
| `ExcelIgnoreUnannotated` | `ignore_unannotated` | `annotation/excel_ignore_unannotated.rs` |
| `ExcelProperty` | `property`, `value/head`, `name`, `index`, `order`, `converter` | `annotation/excel_property.rs` |
| `DateTimeFormat` | `date_time_format`, `use_1904_windowing` | `annotation/format/date_time_format.rs` |
| `NumberFormat` | `number_format`, `rounding_mode` | `annotation/format/number_format.rs` |
| `ColumnWidth` | `column_width` | `annotation/write/style/column_width.rs` |
| `ContentFontStyle` | `content_font_style(...)` | `annotation/write/style/content_font_style.rs` |
| `ContentLoopMerge` | `content_loop_merge(...)` | `annotation/write/style/content_loop_merge.rs` |
| `ContentRowHeight` | `content_row_height` | `annotation/write/style/content_row_height.rs` |
| `ContentStyle` | `content_style(...)` | `annotation/write/style/content_style.rs` |
| `HeadFontStyle` | `head_font_style(...)` | `annotation/write/style/head_font_style.rs` |
| `HeadRowHeight` | `head_row_height` | `annotation/write/style/head_row_height.rs` |
| `HeadStyle` | `head_style(...)` | `annotation/write/style/head_style.rs` |
| `OnceAbsoluteMerge` | `once_absolute_merge(...)` | `annotation/write/style/once_absolute_merge.rs` |

`ExcelProperty.value()` 对应 `value = ["一级", "二级"]`。写入 XLSX、XLS 和 CSV 时保留完整路径，XLS/XLSX 默认执行相邻同名表头合并；读取时与 Java 一致，使用最后一级名称匹配字段。

`ignore_unannotated` 只认可 `property`、`value/head`、`name`、`index`、`order` 或 `converter` 这些 `ExcelProperty` 等价声明。仅配置格式或样式不会让字段自动进入映射，这与 Java `ExcelIgnoreUnannotated` 的判定一致。

`DateTimeFormat` 与 `NumberFormat` 使用独立元数据槽位，因此同一字段同时声明
`date_time_format` 和 `number_format` 时不会互相覆盖。已废弃的
`ExcelProperty.format` 仍可通过 `format` 使用，并仅作为两类格式都未声明时的兼容回退值。

Java 注解中的 `int` 参数按有符号整数解析，`index = -1` 与
`once_absolute_merge(...)` 的 `-1` 哨兵不会通过伪造负数字面量生成代码；列宽和
行高显式设置为 `-1` 时等价于未覆盖默认值。
忽略字段默认仍要求实现 `Default`；对无法实现 `Default` 的字段可以显式指定
`#[excel(ignore, default = expression)]`。`default` 是 EasyExcel-Rust 的派生宏扩展，
并非 Java 注解成员。

`image`、`comment`、`hyperlink`、`formula`、`data_validation`、`conditional` 与
`filter` 也是 EasyExcel-Rust 扩展能力；它们不冒充 Java
`com.alibaba.excel.annotation` 目录中的 14 个注解。Java 对照表只列出语义来源明确的
注解，扩展能力由 `docs/compatibility.md` 单独描述。

注解解析和元数据生成能力是一致的；具体文件格式的渲染边界仍以根目录 `docs/compatibility.md` 为准，例如 BIFF8 的完整 HSSF 调色板与 XF 细节仍属于格式后端限制。
