# Rust 项目规范合规审计报告

**扫描日期**: 2026-08-11
**扫描范围**: 4 个核心 crate（994 个 .rs 文件）
**扫描维度**: 4 维度全覆盖

---

## 总览

| 维度 | 违规文件数 | 违规项总数 |
|------|-----------|-----------|
| 维度 1: 一个 .rs 多个 pub 类型 | 8 | 18 个类型（溢出 10 个） |
| 维度 2: mod.rs/lib.rs 含类型定义 | 0 | 0 |
| 维度 3: wildcard import 于生产代码 | 0 | 0 |
| 维度 4: STUB / 空函数体 | 6 | 93 个 STUB |
| **合计** | **13** | **103** |

> 说明：维度 2 和维度 3 均为零违规。所有 wildcard import 均位于 `#[cfg(test)]` 测试模块内，合规。

---

## Top 10 最严重违规（按 文件行数 x 违规类型数 评分）

| 排名 | 文件 | 行数 | 违规类型 | 违规项数 | 严重度评分 | 说明 |
|------|------|------|---------|---------|-----------|------|
| **1** | `crates/easyexcel-csv/src/csv/csv_sheet.rs` | 384 | STUB | 47 | **384 x 47 = 18048** | impl 方法大面积空实现 |
| **2** | `crates/easyexcel-csv/src/csv/csv_cell_style.rs` | 295 | STUB | 21 | **295 x 21 = 6195** | impl 方法大面积空实现 |
| **3** | `crates/easyexcel-csv/src/csv/csv_workbook.rs` | 362 | STUB | 15 | **362 x 15 = 5430** | impl 方法大面积空实现 |
| **4** | `crates/easyexcel-csv/src/csv/csv_cell.rs` | 306 | 多类型 + STUB | 2 + 3 | **306 x 5 = 1530** | 两种违规叠加 |
| **5** | `crates/easyexcel-xlsx/src/xlsx/template_fill/template_hyperlink.rs` | 158 | 多类型 | 3 | **158 x 3 = 474** | 1 enum + 2 struct |
| **6** | `crates/easyexcel/src/context/analysis_context.rs` | 183 | 多类型 | 2 | **183 x 2 = 366** | 1 trait + 1 struct |
| **7** | `crates/easyexcel/src/write/metadata/fill/fill_config.rs` | 175 | 多类型 | 2 | **175 x 2 = 350** | 2 struct |
| **8** | `crates/easyexcel/src/event/abstract_ignore_exception_read_listener.rs` | 168 | 多类型 | 2 | **168 x 2 = 336** | 1 trait + 1 struct |
| **9** | `crates/easyexcel/src/event/analysis_event_listener.rs` | 148 | 多类型 | 2 | **148 x 2 = 296** | 1 trait + 1 struct |
| **10** | `crates/easyexcel/src/read/listener/ignore_exception_read_listener.rs` | 123 | 多类型 | 2 | **123 x 2 = 246** | 1 trait + 1 struct |

---

## 维度 1 详细清单：一个 .rs 文件含多个 pub 类型

### 规则
> 一个 .rs 文件只对应一个 Java 对象（类/接口/枚举/record 各一个文件）

### 违规文件（8 个）

#### 1.1 `crates/easyexcel-xlsx/src/xlsx/template_fill/template_hyperlink.rs` (158 行)
- **违规类型数**: 3
- **违规内容**:
  - L3: `pub enum TemplateHyperlinkType` (枚举)
  - L56: `pub struct TemplateHyperlinkCoordinate` (结构体)
  - L65: `pub struct TemplateHyperlink` (结构体)
- **修复建议**: 拆分为 3 个文件:
  - `template_hyperlink_type.rs` (enum)
  - `template_hyperlink_coordinate.rs` (struct)
  - `template_hyperlink.rs` (struct)

#### 1.2 `crates/easyexcel-csv/src/csv/csv_cell.rs` (306 行)
- **违规类型数**: 2
- **违规内容**:
  - L16: `pub enum CsvCellType` (枚举)
  - L36: `pub struct CsvCell` (结构体)
- **修复建议**: 拆分为 2 个文件:
  - `csv_cell_type.rs` (enum)
  - `csv_cell.rs` (struct, 保留原位)

#### 1.3 `crates/easyexcel/src/context/analysis_context.rs` (183 行)
- **违规类型数**: 2
- **违规内容**:
  - L15: `pub trait AnalysisContextLifecycle` (trait)
  - L99: `pub struct AnalysisContext` (结构体)
- **修复建议**: 拆分为 2 个文件:
  - `analysis_context_lifecycle.rs` (trait)
  - `analysis_context.rs` (struct, 保留原位)

#### 1.4 `crates/easyexcel/src/write/metadata/fill/fill_config.rs` (175 行)
- **违规类型数**: 2
- **违规内容**:
  - L9: `pub struct FillConfig` (结构体)
  - L136: `pub struct FillConfigBuilder` (结构体)
- **修复建议**: 拆分为 2 个文件:
  - `fill_config.rs` (struct, 保留原位)
  - `fill_config_builder.rs` (struct)

#### 1.5 `crates/easyexcel/src/event/abstract_ignore_exception_read_listener.rs` (168 行)
- **违规类型数**: 2
- **违规内容**:
  - L9: `pub trait AbstractIgnoreExceptionReadListener<T>` (trait)
  - L36: `pub struct AbstractIgnoreExceptionListenerAdapter<L>` (结构体)
- **修复建议**: 拆分为 2 个文件:
  - `abstract_ignore_exception_read_listener.rs` (trait, 保留原位)
  - `abstract_ignore_exception_listener_adapter.rs` (struct)

#### 1.6 `crates/easyexcel/src/event/analysis_event_listener.rs` (148 行)
- **违规类型数**: 2
- **违规内容**:
  - L8: `pub trait AnalysisEventListener<T>` (trait)
  - L31: `pub struct AnalysisEventListenerAdapter<L>` (结构体)
- **修复建议**: 拆分为 2 个文件:
  - `analysis_event_listener.rs` (trait, 保留原位)
  - `analysis_event_listener_adapter.rs` (struct)

#### 1.7 `crates/easyexcel/src/read/listener/ignore_exception_read_listener.rs` (123 行)
- **违规类型数**: 2
- **违规内容**:
  - L12: `pub trait IgnoreExceptionReadListener<T>` (trait)
  - L36: `pub struct IgnoreExceptionListenerAdapter<L>` (结构体)
- **修复建议**: 拆分为 2 个文件:
  - `ignore_exception_read_listener.rs` (trait, 保留原位)
  - `ignore_exception_listener_adapter.rs` (struct)

#### 1.8 `crates/easyexcel-xlsx/src/xlsx/template_fill/template_image.rs` (54 行)
- **违规类型数**: 2
- **违规内容**:
  - L3: `pub enum TemplateImageMovement` (枚举)
  - L14: `pub struct TemplateImage` (结构体)
- **修复建议**: 拆分为 2 个文件:
  - `template_image_movement.rs` (enum)
  - `template_image.rs` (struct, 保留原位)

---

## 维度 2: mod.rs/lib.rs 含类型定义

**结果: 0 违规**

所有 `mod.rs` 和 `lib.rs` 文件均仅包含 `mod` 声明和 `pub use` 重导出，无类型定义。合规。

---

## 维度 3: wildcard import 于生产代码

**结果: 0 违规**

扫描发现的所有 `use xxx::*` 均位于 `#[cfg(test)]` 测试模块内，包括:
- `crates/easyexcel/src/tests.rs`
- `crates/easyexcel/src/read/tests.rs`
- `crates/easyexcel/src/template/tests.rs`
- `crates/easyexcel/src/write/tests.rs`
- `crates/easyexcel/src/core/tests.rs`
- 以及其他 `*_tests/tests.rs` 文件

`crates/easyexcel/src/util/mod.rs:36` 有一行注释说明刻意不使用 wildcard import，合规。

---

## 维度 4: STUB / 空函数体 详细清单

### 规则
> 禁止 STUB 充数（函数体为空 / `unimplemented!()` / `todo!()`）

### 违规文件（6 个，共 93 个 STUB）

#### 4.1 `crates/easyexcel-csv/src/csv/csv_sheet.rs` (384 行) -- **47 个 STUB**

全部为 `impl CsvSheet` 中的空方法体 `{}`。典型示例:
- L262: `pub const fn create_freeze_pane(&mut self, _column_split: usize, _row_split: usize) {}`
- L265: `pub const fn set_zoom(&mut self, _scale: usize) {}`
- L291-335: `set_default_column_width`, `set_default_row_height`, `set_horizontally_center`, `set_vertically_center`, `set_display_zeros`, `set_display_formulas`, `set_print_gridlines`, `set_selected`, `set_right_to_left`, `set_force_formula_recalculation`, `shift_rows`, `shift_columns`, `remove_merged_region`, `remove_merged_regions`, `validate_merged_regions`, `set_column_break`, `remove_column_break`, `set_row_break`, `remove_row_break`, `group_column`, `ungroup_column`, `group_row`, `ungroup_row`, `set_column_group_collapsed`, `set_row_group_collapsed`, `set_column_hidden`, `set_default_column_style`, `auto_size_column`, `create_split_pane`, `show_in_pane`
- L348-369: `set_display_gridlines`, `set_display_row_col_headings`, `set_print_row_and_column_headings`, `set_autobreaks`, `set_display_guts`, `set_fit_to_page`, `set_row_sums_below`, `set_row_sums_right`, `set_margin`, `set_auto_filter`, `set_repeating_columns`, `set_repeating_rows`, `set_active_cell`, `add_validation_data`

**说明**: 这些方法的注释标注 "CSV 不保存 xxx；保留 Java no-op 调用体验"。虽然是有意为之（CSV 格式不支持这些 Excel 功能），但仍违反 STUB 禁止规则。
**修复建议**: 对 CSV 不支持的功能，应返回明确的 `Err(UnsupportedFeature)` 或使用 `#[allow(unused)]` 标注，而非静默空实现。

#### 4.2 `crates/easyexcel-csv/src/csv/csv_cell_style.rs` (295 行) -- **21 个 STUB**

全部为 `impl CsvCellStyle` 中的空方法体。涉及样式相关方法:
- L223-264: `set_hidden`, `set_locked`, `set_quote_prefixed`, `set_wrap_text`, `set_rotation`, `set_indention`, `set_shrink_to_fit`, `set_alignment`, `set_vertical_alignment`, `set_border_left`, `set_border_right`, `set_border_top`, `set_border_bottom`, `set_left_border_color`, `set_right_border_color`, `set_top_border_color`, `set_bottom_border_color`, `set_fill_pattern`, `set_fill_background_color`, `set_fill_foreground_color`, `clone_style_from`

**修复建议**: 同上，返回 `Err(UnsupportedFeature)` 或记录 warning 日志。

#### 4.3 `crates/easyexcel-csv/src/csv/csv_workbook.rs` (362 行) -- **15 个 STUB**

全部为 `impl CsvWorkbook` 中的空方法体:
- L286-293: `set_active_sheet`, `set_first_visible_tab`, `set_selected_tab`, `set_sheet_order`, `set_sheet_name`, `set_hidden`, `set_sheet_hidden`, `set_force_formula_recalculation`
- L321-324: `set_sheet_visibility`, `set_print_area`, `remove_print_area`, `flush_data`
- L350-353: `remove_name`, `set_missing_cell_policy`, `add_tool_pack`

**修复建议**: 同上。

#### 4.4 `crates/easyexcel-csv/src/csv/csv_cell.rs` (306 行) -- **3 个 STUB**

- L284: `fn remove_cell_comment(&mut self) {}`
- L289: `fn remove_hyperlink(&mut self) {}`
- L296: `fn set_as_active_cell(&mut self) {}`

**修复建议**: 同上。

#### 4.5 `crates/easyexcel-csv/src/csv/csv_row.rs` (267 行) -- **2 个 STUB**

- L208: `fn shift_cells_right(&mut self, _start_col: u16, _count: u16) {}`
- L217: `fn shift_cells_left(&mut self, _start_col: u16, _count: u16) {}`

**修复建议**: 同上。

#### 4.6 `crates/easyexcel-csv/src/csv/csv_rich_text_string.rs` (91 行) -- **2 个 STUB**

- L63: `fn apply_font(&mut self, _font: &dyn Font, _start_index: u32, _end_index: u32) {}`
- L66: `fn clear_formatting(&mut self) {}`

**修复建议**: 同上。

#### 4.7 `crates/easyexcel/src/read/listener/model_build_event_listener.rs` (54 行) -- **1 个 STUB**

- L53: `pub const fn do_after_all_analysed(&mut self) {}`

**说明**: 注释标注 "Java doAfterAllAnalysed 的空实现"。这是一个默认空实现，子类可覆盖。
**修复建议**: 若为默认实现，可保留但需添加 `#[allow(clippy::unused_self)]` 注解和更详细的文档说明。

---

## 未发现违规的说明

### 已排除的误报

| 文件 | 排除原因 |
|------|---------|
| `crates/easyexcel/src/analysis/v03/xls_record_handler.rs` | `fn process_record(...)` 为 trait 方法声明（以 `;` 结尾），非 STUB |
| `crates/easyexcel/src/analysis/v03/ignorable_xls_record_handler.rs` | `fn assert_java_marker` 位于 `#[cfg(test)] mod tests` 内，非生产代码 |
| 所有 trait 方法声明（以 `;` 结尾） | 抽象方法声明，非 STUB 实现 |
| 所有 `Default::default()` / `From::from()` 实现 | 有实际逻辑的一行实现，非空 STUB |

### 无 compat.rs 文件
扫描范围内不存在 `compat.rs` 文件。

---

## 建议修复顺序

### 第一优先级：STUB 清理（CSV crate，93 个 STUB）
CSV crate 集中了全部 90 个 STUB（占 97%）。建议:
1. `csv_sheet.rs` (47 个) -- 最大面，优先处理
2. `csv_cell_style.rs` (21 个)
3. `csv_workbook.rs` (15 个)
4. `csv_cell.rs` (3 个)
5. `csv_row.rs` (2 个)
6. `csv_rich_text_string.rs` (2 个)

统一方案: 对 CSV 不支持的功能方法，返回 `Err(ExcelError::Unsupported("CSV 格式不支持此功能"))` 而非静默空实现。

### 第二优先级：文件拆分（8 个文件，18 个类型）
按严重度排序:
1. `template_hyperlink.rs` (3 类型 -> 3 文件)
2. `csv_cell.rs` (2 类型 -> 2 文件)
3. `analysis_context.rs` (2 类型 -> 2 文件)
4. `fill_config.rs` (2 类型 -> 2 文件)
5. `abstract_ignore_exception_read_listener.rs` (2 类型 -> 2 文件)
6. `analysis_event_listener.rs` (2 类型 -> 2 文件)
7. `ignore_exception_read_listener.rs` (2 类型 -> 2 文件)
8. `template_image.rs` (2 类型 -> 2 文件)

### 第三优先级：零散 STUB
- `model_build_event_listener.rs` (1 个) -- 评估是否保留默认空实现

---

## 统计摘要

- **扫描 crate 数**: 4
- **扫描文件数**: 994
- **违规文件数**: 13（去重后，csv_cell.rs 同时违反维度 1 和维度 4）
- **维度 1 违规**: 8 个文件，18 个 pub 类型需拆分
- **维度 2 违规**: 0
- **维度 3 违规**: 0
- **维度 4 违规**: 6 个文件，93 个 STUB 空实现
- **总违规项**: 103
