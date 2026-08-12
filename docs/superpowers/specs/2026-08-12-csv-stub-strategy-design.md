# easyexcel-csv STUB 处置策略

## 1. 背景

`easyexcel-csv` crate 实现了 Java EasyExcel 的 CSV 后端。由于 CSV 格式本身不支持 Excel 的许多高级特性（样式、合并区域、冻结窗格等），Rust 实现中保留了大量"空实现"函数，以维持与 Java API 的调用兼容性。

Agent 41 审计（commit 3c4086c）首次系统性识别了这些 STUB 函数。

## 2. STUB 统计

### 2.1 按文件分布

| 文件 | STUB 函数数 | 总 pub 函数数 | STUB 占比 |
|------|-------------|---------------|-----------|
| `csv_sheet.rs` | 112 | 143 | 78% |
| `csv_cell_style.rs` | 61 | 80 | 76% |
| `csv_workbook.rs` | 56 | 96 | 58% |
| `csv_cell.rs` | 40 | ~80 | 50% |
| **合计** | **269** | **~399** | **67%** |

> 注：STUB 定义为函数体为空（`{}`）或仅返回固定值（`0`/`false`/`None`/空 `Vec`）的 no-op 实现。

### 2.2 按 Java 功能分类

#### CsvSheet（csv_sheet.rs）— 112 STUB

| 功能域 | STUB 数 | 典型函数 |
|--------|---------|----------|
| 合并区域 (Merged Region) | 8 | `add_merged_region`, `number_of_merged_regions`, `remove_merged_region` |
| 列宽/隐藏 (Column Width/Hidden) | 3 | `column_width`, `is_column_hidden`, `set_column_hidden` |
| 冻结窗格 (Freeze Pane) | 3 | `create_freeze_pane`, `create_split_pane`, `show_in_pane` |
| 缩放 (Zoom) | 2 | `set_zoom`, `get_zoom` |
| 公式重算 (Formula Recalc) | 2 | `force_formula_recalculation`, `set_force_formula_recalculation` |
| 视图属性 (View Properties) | 24 | `get_default_column_width`, `is_display_zeros`, `is_selected` |
| 视图 Setter | 11 | `set_default_column_width`, `set_display_zeros`, `set_selected` |
| 行操作 (Row Operations) | 2 | `shift_rows`, `shift_columns` |
| 列样式/轮廓/分页符 | 20 | `get_column_style`, `get_column_outline_level`, `set_column_break` |
| 打印/显示状态 | 25 | `is_display_gridlines`, `get_autobreaks`, `set_auto_filter` |
| 批注/超链接/验证 | 12 | `get_cell_comments`, `get_hyperlink_list`, `get_data_validations` |

#### CsvCellStyle（csv_cell_style.rs）— 61 STUB

| 功能域 | STUB 数 | 典型函数 |
|--------|---------|----------|
| 字体 (Font) | 2 | `font_index`, `set_font` |
| 单元格属性 | 7 | `hidden`, `locked`, `quote_prefixed`, `wrap_text`, `rotation`, `indention`, `shrink_to_fit` |
| 对齐 (Alignment) | 2 | `alignment`, `vertical_alignment` |
| 边框 (Border) | 8 | `border_left`, `border_right`, `border_top`, `border_bottom`, `left_border_color` |
| 填充 (Fill) | 3 | `fill_pattern`, `fill_background_color`, `fill_foreground_color` |
| Setter（no-op） | 20 | `set_hidden`, `set_locked`, `set_alignment`, `set_border_left` |
| 克隆 (Clone) | 1 | `clone_style_from` |

#### CsvWorkbook（csv_workbook.rs）— 56 STUB

| 功能域 | STUB 数 | 典型函数 |
|--------|---------|----------|
| 字体/名称表 | 2 | `number_of_fonts`, `number_of_names` |
| 隐藏状态 | 1 | `is_hidden` |
| 公式重算 | 2 | `force_formula_recalculation`, `set_force_formula_recalculation` |
| 工作表导航 | 8 | `get_active_sheet_index`, `set_active_sheet`, `set_sheet_order` |
| 工作表可见性 | 6 | `is_sheet_hidden`, `set_sheet_hidden`, `get_sheet_visibility` |
| 缺失单元格策略 | 2 | `get_missing_cell_policy`, `set_missing_cell_policy` |
| 名称管理 | 3 | `get_all_names`, `get_names`, `remove_name` |
| 图片 | 1 | `get_all_pictures` |
| 打印区域 | 3 | `get_print_area`, `set_print_area`, `remove_print_area` |
| 字体查询 | 2 | `get_font_at`, `find_font` |
| 其他 | 26 | `get_spreadsheet_version`, `flush_data`, `add_tool_pack` |

#### CsvCell（csv_cell.rs）— 40 STUB

| 功能域 | STUB 数 | 典型函数 |
|--------|---------|----------|
| 批注 (Comment) | 3 | `remove_cell_comment`, `get_cell_comment`, `set_cell_comment` |
| 超链接 (Hyperlink) | 3 | `remove_hyperlink`, `get_hyperlink`, `set_hyperlink` |
| 数组公式 | 3 | `get_array_formula_range`, `is_part_of_array_formula_group` |
| 活动单元格 | 1 | `set_as_active_cell` |

## 3. 三种处置方案对比

### 方案 A：保留现状（Status Quo）

**做法**：维持当前 no-op 实现，注释标注"Java no-op 调用体验"。

**优点**：
- 零改动成本
- Java 调用方无需修改代码（drop-in 替换）
- 编译时零开销（`const fn` 优化）

**缺点**：
- 调用方无法区分"功能不支持"与"功能正常但返回默认值"
- 269 个空函数增加代码体积和维护负担
- 违反 Rust "显式错误"哲学
- 新开发者可能误以为这些功能已实现

### 方案 B：改为 `Err(UnsupportedFeature)`

**做法**：所有 STUB 函数返回 `Result`，错误类型为 `UnsupportedFeature`。

**优点**：
- 显式告知调用方功能不可用
- 符合 Rust 错误处理最佳实践
- 调用方可以在运行时优雅降级

**缺点**：
- **破坏性 API 变更**：所有调用点需要处理 `Result`
- 实现工作量大（269 个函数签名变更）
- Java 兼容性丧失（Java 侧是 void/no-op，不是异常）
- 需要引入新的错误类型

### 方案 C：拆出独立 stub 模块（推荐）

**做法**：将所有 STUB 函数集中到 `easyexcel-csv/src/stubs/` 目录，按 Java 包/功能分类，原文件通过 `pub use` 重导出。

**优点**：
- 主业务代码更清晰（STUB 与实现分离）
- STUB 集中管理，便于未来批量处理
- 公共 API 不变（`pub use` 重导出）
- 可以逐步迁移：先集中，再逐个决定处置

**缺点**：
- 需要重构工作（约 4-6h）
- 模块层级变深

## 4. 推荐方案

**推荐方案 C（拆出独立 stub 模块）**，理由如下：

1. **零破坏性**：`pub use` 重导出保持公共 API 不变
2. **渐进式**：先集中，再逐个评估是否改为 `Err(UnsupportedFeature)`
3. **可维护性**：STUB 集中后，可以批量添加 `#[deprecated]` 或统一错误返回
4. **符合 Rust 生态惯例**：`tokio`、`serde` 等crate 也采用类似模式处理可选功能

### 4.1 目录结构设计

```
easyexcel-csv/src/stubs/
├── mod.rs                    # pub use 重导出
├── sheet_stubs.rs            # CsvSheet 的 STUB 方法
├── cell_style_stubs.rs       # CsvCellStyle 的 STUB 方法
├── workbook_stubs.rs         # CsvWorkbook 的 STUB 方法
└── cell_stubs.rs             # CsvCell 的 STUB 方法
```

### 4.2 每个 STUB 应放哪个文件

| Java 包/功能 | Rust 文件 | STUB 数 |
|--------------|-----------|---------|
| `com.alibaba.excel.metadata.csv.CsvSheet` | `sheet_stubs.rs` | 112 |
| `com.alibaba.excel.metadata.csv.CsvCellStyle` | `cell_style_stubs.rs` | 61 |
| `com.alibaba.excel.metadata.csv.CsvWorkbook` | `workbook_stubs.rs` | 56 |
| `com.alibaba.excel.metadata.csv.CsvCell` | `cell_stubs.rs` | 40 |

### 4.3 实施步骤

1. 创建 `easyexcel-csv/src/stubs/` 目录和 `mod.rs`
2. 为每个 STUB 函数创建独立的 trait（如 `CsvSheetStubs`）
3. 在原类型上 `impl` 这些 trait
4. 在 `mod.rs` 中 `pub use` 重导出
5. 验证编译通过

### 4.4 实施估算

| 阶段 | 工作量 | 说明 |
|------|--------|------|
| 创建目录和 mod.rs | 0.5h | 模板代码 |
| 迁移 CsvSheet STUB | 1.5h | 112 个函数 |
| 迁移 CsvCellStyle STUB | 1h | 61 个函数 |
| 迁移 CsvWorkbook STUB | 1h | 56 个函数 |
| 迁移 CsvCell STUB | 0.5h | 40 个函数 |
| 测试验证 | 0.5h | cargo check + cargo test |
| **合计** | **~5h** | |

## 5. csv_cell.rs 拆分记录

本次任务同时完成了 `csv_cell.rs` 的拆分：

### 5.1 拆分前

```
csv/csv_cell.rs              # 使用 include!() 宏内联两个子文件
csv/csv_cell/csv_numeric_cell_type.rs  # 被 include!() 引入
csv/csv_cell/csv_cell_value.rs         # 被 include!() 引入
```

问题：`csv_cell.rs` 包含 4 个 pub 类型（`CsvCell`, `CsvCellType`, `CsvCellValue`, `CsvNumericCellType`），违反"一个 .rs 文件只对应一个 Java 对象"规范。

### 5.2 拆分后

```
csv/csv_cell/mod.rs                    # mod 声明 + pub use 重导出
csv/csv_cell/csv_cell.rs               # CsvCell 结构体（对应 Java CsvCell）
csv/csv_cell/csv_cell_type.rs          # CsvCellType 枚举（对应 Java CellType）
csv/csv_cell/csv_cell_value.rs         # CsvCellValue trait（Rust 架构扩展）
csv/csv_cell/csv_numeric_cell_type.rs  # CsvNumericCellType 枚举（Rust 架构扩展）
```

### 5.3 验证

`cargo check -p easyexcel-csv --lib` 通过，仅有 pre-existing 的 missing-docs 警告。

## 6. 后续建议

1. **短期**：实施方案 C，将 STUB 集中到 `stubs/` 模块
2. **中期**：为 STUB 函数添加 `#[deprecated(note = "CSV 不支持此功能")]` 属性
3. **长期**：评估是否将部分 STUB 改为 `Err(UnsupportedFeature)`，特别是样式相关函数
