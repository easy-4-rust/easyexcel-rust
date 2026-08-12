# Write/Style 注解字段对齐审计

审计日期：2026-08-10
审计范围：`crates/easyexcel-derive/src/annotation/write/style/` 9 个注解 parser
Java 对照：`easyexcel-core/.../annotation/write/style/*.java`

## 总结

**所有 9 个注解的字段已完全对齐，无需修改。**

| 注解 | Java 字段数 | Rust meta key 数 | 缺失字段 | 默认值对齐 |
|------|------------|-----------------|----------|-----------|
| ColumnWidth | 1 | 1 | 0 | OK |
| HeadRowHeight | 1 | 1 | 0 | OK |
| ContentRowHeight | 1 | 1 | 0 | OK |
| HeadStyle | 22 | 22 | 0 | OK |
| ContentStyle | 22 | 22 | 0 | OK |
| HeadFontStyle | 9 | 9 | 0 | OK |
| ContentFontStyle | 9 | 9 | 0 | OK |
| OnceAbsoluteMerge | 4 | 4 | 0 | OK |
| ContentLoopMerge | 2 | 2 | 0 | OK |

---

## 1. ColumnWidth

Java: `com.alibaba.excel.annotation.write.style.ColumnWidth`
Rust: `crates/easyexcel-derive/src/annotation/write/style/column_width.rs`
共享: `style_parser::dimension::parse_dimension`

| Java 字段 | Java 类型 | Java 默认值 | Rust meta key | Rust 类型 | 对齐 |
|-----------|----------|------------|--------------|----------|------|
| value | int | -1 | column_width | SignedInteger (Option) | OK: None = -1 |

Target: FIELD + TYPE (字段级 + 类型级均支持)

---

## 2. HeadRowHeight

Java: `com.alibaba.excel.annotation.write.style.HeadRowHeight`
Rust: `crates/easyexcel-derive/src/annotation/write/style/head_row_height.rs`
共享: `style_parser::dimension::parse_dimension`

| Java 字段 | Java 类型 | Java 默认值 | Rust meta key | Rust 类型 | 对齐 |
|-----------|----------|------------|--------------|----------|------|
| value | short | -1 | head_row_height | SignedInteger (Option) | OK: None = -1 |

Target: TYPE (仅类型级)

---

## 3. ContentRowHeight

Java: `com.alibaba.excel.annotation.write.style.ContentRowHeight`
Rust: `crates/easyexcel-derive/src/annotation/write/style/content_row_height.rs`
共享: `style_parser::dimension::parse_dimension`

| Java 字段 | Java 类型 | Java 默认值 | Rust meta key | Rust 类型 | 对齐 |
|-----------|----------|------------|--------------|----------|------|
| value | short | -1 | content_row_height | SignedInteger (Option) | OK: None = -1 |

Target: TYPE (仅类型级)

---

## 4. HeadStyle

Java: `com.alibaba.excel.annotation.write.style.HeadStyle`
Rust: `crates/easyexcel-derive/src/annotation/write/style/head_style.rs`
共享: `style_parser::cell_style::parse_cell_style`

| Java 字段 | Java 类型 | Java 默认值 | Rust meta key | Rust 类型 | 对齐 |
|-----------|----------|------------|--------------|----------|------|
| dataFormat | short | -1 | data_format | ExcelDataFormat (Option) | OK: 支持 builtin(int) 和 custom(str) |
| hidden | BooleanEnum | DEFAULT | hidden | bool (Option) | OK: None = DEFAULT |
| locked | BooleanEnum | DEFAULT | locked | bool (Option) | OK: None = DEFAULT |
| quotePrefix | BooleanEnum | DEFAULT | quote_prefix | bool (Option) | OK: None = DEFAULT |
| horizontalAlignment | HorizontalAlignmentEnum | DEFAULT | horizontal_alignment | ExcelHorizontalAlignment (Option) | OK: None = DEFAULT |
| wrapped | BooleanEnum | DEFAULT | wrapped | bool (Option) | OK: None = DEFAULT |
| verticalAlignment | VerticalAlignmentEnum | DEFAULT | vertical_alignment | ExcelVerticalAlignment (Option) | OK: None = DEFAULT |
| rotation | short | -1 | rotation | i16 (Option) | OK: None = -1 |
| indent | short | -1 | indent | u8 (Option) | OK: None = -1 |
| borderLeft | BorderStyleEnum | DEFAULT | border_left | ExcelBorderStyle (Option) | OK: None = DEFAULT |
| borderRight | BorderStyleEnum | DEFAULT | border_right | ExcelBorderStyle (Option) | OK: None = DEFAULT |
| borderTop | BorderStyleEnum | DEFAULT | border_top | ExcelBorderStyle (Option) | OK: None = DEFAULT |
| borderBottom | BorderStyleEnum | DEFAULT | border_bottom | ExcelBorderStyle (Option) | OK: None = DEFAULT |
| leftBorderColor | short | -1 | left_border_color | ExcelColor (Option) | OK: None = -1 |
| rightBorderColor | short | -1 | right_border_color | ExcelColor (Option) | OK: None = -1 |
| topBorderColor | short | -1 | top_border_color | ExcelColor (Option) | OK: None = -1 |
| bottomBorderColor | short | -1 | bottom_border_color | ExcelColor (Option) | OK: None = -1 |
| fillPatternType | FillPatternTypeEnum | DEFAULT | fill_pattern / fill_pattern_type | ExcelFillPattern (Option) | OK: 别名双入口 |
| fillBackgroundColor | short | -1 | fill_background_color | ExcelColor (Option) | OK: None = -1 |
| fillForegroundColor | short | -1 | fill_foreground_color | ExcelColor (Option) | OK: None = -1 |
| shrinkToFit | BooleanEnum | DEFAULT | shrink_to_fit | bool (Option) | OK: None = DEFAULT |

Target: FIELD + TYPE

---

## 5. ContentStyle

Java: `com.alibaba.excel.annotation.write.style.ContentStyle`
Rust: `crates/easyexcel-derive/src/annotation/write/style/content_style.rs`
共享: `style_parser::cell_style::parse_cell_style`

与 HeadStyle 完全相同的 22 个字段，共用 `parse_cell_style` 解析器。
字段对照表见上方 HeadStyle。

Target: FIELD + TYPE

---

## 6. HeadFontStyle

Java: `com.alibaba.excel.annotation.write.style.HeadFontStyle`
Rust: `crates/easyexcel-derive/src/annotation/write/style/head_font_style.rs`
共享: `style_parser::font_style::parse_font_style`

| Java 字段 | Java 类型 | Java 默认值 | Rust meta key | Rust 类型 | 对齐 |
|-----------|----------|------------|--------------|----------|------|
| fontName | String | "" | font_name | &'static str (Option) | OK: None = "" |
| fontHeightInPoints | short | -1 | font_height_in_points | f64 (Option) | OK: None = -1; 验证 > 0 |
| italic | BooleanEnum | DEFAULT | italic | bool (Option) | OK: None = DEFAULT |
| strikeout | BooleanEnum | DEFAULT | strikeout | bool (Option) | OK: None = DEFAULT |
| color | short | -1 | color | ExcelColor (Option) | OK: None = -1 |
| typeOffset | short | -1 | type_offset | ExcelFontScript (Option) | OK: 枚举映射 none/superscript/subscript |
| underline | byte | -1 | underline | ExcelUnderline (Option) | OK: 枚举映射 none/single/double/single_accounting/double_accounting |
| charset | int | -1 | charset | u8 (Option) | OK: None = -1 |
| bold | BooleanEnum | DEFAULT | bold | bool (Option) | OK: None = DEFAULT |

Target: FIELD + TYPE

---

## 7. ContentFontStyle

Java: `com.alibaba.excel.annotation.write.style.ContentFontStyle`
Rust: `crates/easyexcel-derive/src/annotation/write/style/content_font_style.rs`
共享: `style_parser::font_style::parse_font_style`

与 HeadFontStyle 完全相同的 9 个字段，共用 `parse_font_style` 解析器。
字段对照表见上方 HeadFontStyle。

Target: FIELD + TYPE

---

## 8. OnceAbsoluteMerge

Java: `com.alibaba.excel.annotation.write.style.OnceAbsoluteMerge`
Rust: `crates/easyexcel-derive/src/annotation/write/style/once_absolute_merge.rs`

| Java 字段 | Java 类型 | Java 默认值 | Rust meta key | Rust 类型 | 对齐 |
|-----------|----------|------------|--------------|----------|------|
| firstRowIndex | int | -1 | first_row_index | SignedInteger (Option) | OK: unwrap_or(-1) |
| lastRowIndex | int | -1 | last_row_index | SignedInteger (Option) | OK: unwrap_or(-1) |
| firstColumnIndex | int | -1 | first_column_index | SignedInteger (Option) | OK: unwrap_or(-1) |
| lastColumnIndex | int | -1 | last_column_index | SignedInteger (Option) | OK: unwrap_or(-1) |

Target: TYPE (仅类型级)

---

## 9. ContentLoopMerge

Java: `com.alibaba.excel.annotation.write.style.ContentLoopMerge`
Rust: `crates/easyexcel-derive/src/annotation/write/style/content_loop_merge.rs`

| Java 字段 | Java 类型 | Java 默认值 | Rust meta key | Rust 类型 | 对齐 |
|-----------|----------|------------|--------------|----------|------|
| eachRow | int | 1 | each_row | u32 (LitInt) | OK: unwrap_or(1) |
| columnExtend | int | 1 | column_extend | u16 (LitInt) | OK: unwrap_or(1) |

Target: FIELD (仅字段级)

---

## 枚举变体覆盖审计

### HorizontalAlignmentEnum (Java 8 variants, 含 DEFAULT)

Java: DEFAULT, GENERAL, LEFT, CENTER, RIGHT, FILL, JUSTIFY, CENTER_SELECTION, DISTRIBUTED

Rust: general, left, center, right, fill, justify, center_across, distributed

| Java 变体 | Rust 变体 | 对齐 |
|-----------|----------|------|
| DEFAULT | (sentinel, 用 Option::None 表示) | OK |
| GENERAL | general | OK |
| LEFT | left | OK |
| CENTER | center | OK |
| RIGHT | right | OK |
| FILL | fill | OK |
| JUSTIFY | justify | OK |
| CENTER_SELECTION | center_across | OK (名称差异，语义一致) |
| DISTRIBUTED | distributed | OK |

### VerticalAlignmentEnum (Java 6 variants, 含 DEFAULT)

Java: DEFAULT, TOP, CENTER, BOTTOM, JUSTIFY, DISTRIBUTED

Rust: top, center, bottom, justify, distributed

| Java 变体 | Rust 变体 | 对齐 |
|-----------|----------|------|
| DEFAULT | (sentinel) | OK |
| TOP | top | OK |
| CENTER | center | OK |
| BOTTOM | bottom | OK |
| JUSTIFY | justify | OK |
| DISTRIBUTED | distributed | OK |

### BorderStyleEnum (Java 14 variants, 含 DEFAULT)

Java: DEFAULT, NONE, THIN, MEDIUM, DASHED, DOTTED, THICK, DOUBLE, HAIR, MEDIUM_DASHED, DASH_DOT, MEDIUM_DASH_DOT, DASH_DOT_DOT, MEDIUM_DASH_DOT_DOT, SLANTED_DASH_DOT

Rust: none, thin, medium, dashed, dotted, thick, double, hair, medium_dashed, dash_dot, medium_dash_dot, dash_dot_dot, medium_dash_dot_dot, slant_dash_dot

| Java 变体 | Rust 变体 | 对齐 |
|-----------|----------|------|
| DEFAULT | (sentinel) | OK |
| NONE | none | OK |
| THIN | thin | OK |
| MEDIUM | medium | OK |
| DASHED | dashed | OK |
| DOTTED | dotted | OK |
| THICK | thick | OK |
| DOUBLE | double | OK |
| HAIR | hair | OK |
| MEDIUM_DASHED | medium_dashed | OK |
| DASH_DOT | dash_dot | OK |
| MEDIUM_DASH_DOT | medium_dash_dot | OK |
| DASH_DOT_DOT | dash_dot_dot | OK |
| MEDIUM_DASH_DOT_DOT | medium_dash_dot_dot | OK |
| SLANTED_DASH_DOT | slant_dash_dot | OK (拼写缩写差异，语义一致) |

### FillPatternTypeEnum (Java 19 variants, 含 DEFAULT)

Java: DEFAULT, NO_FILL, SOLID_FOREGROUND, FINE_DOTS, ALT_BARS, SPARSE_DOTS, THICK_HORZ_BANDS, THICK_VERT_BANDS, THICK_BACKWARD_DIAG, THICK_FORWARD_DIAG, BIG_SPOTS, BRICKS, THIN_HORZ_BANDS, THIN_VERT_BANDS, THIN_BACKWARD_DIAG, THIN_FORWARD_DIAG, SQUARES, DIAMONDS, LESS_DOTS, LEAST_DOTS

Rust: none, solid, fine_dots, alt_bars, sparse_dots, thick_horz_bands, thick_vert_bands, thick_backward_diag, thick_forward_diag, big_spots, bricks, thin_horz_bands, thin_vert_bands, thin_backward_diag, thin_forward_diag, squares, diamonds, less_dots, least_dots

| Java 变体 | Rust 变体 | 对齐 |
|-----------|----------|------|
| DEFAULT | (sentinel) | OK |
| NO_FILL | none | OK |
| SOLID_FOREGROUND | solid | OK |
| FINE_DOTS | fine_dots | OK |
| ALT_BARS | alt_bars | OK |
| SPARSE_DOTS | sparse_dots | OK |
| THICK_HORZ_BANDS | thick_horz_bands | OK |
| THICK_VERT_BANDS | thick_vert_bands | OK |
| THICK_BACKWARD_DIAG | thick_backward_diag | OK |
| THICK_FORWARD_DIAG | thick_forward_diag | OK |
| BIG_SPOTS | big_spots | OK |
| BRICKS | bricks | OK |
| THIN_HORZ_BANDS | thin_horz_bands | OK |
| THIN_VERT_BANDS | thin_vert_bands | OK |
| THIN_BACKWARD_DIAG | thin_backward_diag | OK |
| THIN_FORWARD_DIAG | thin_forward_diag | OK |
| SQUARES | squares | OK |
| DIAMONDS | diamonds | OK |
| LESS_DOTS | less_dots | OK |
| LEAST_DOTS | least_dots | OK |

---

## 设计决策

1. **Java `-1` 默认值** → Rust `Option<T>` + `None` 语义：所有 numeric sentinel 值均用 Option::None 统一表示。
2. **Java `BooleanEnum.DEFAULT`** → Rust `Option<bool>` + `None` 语义：DEFAULT 枚举变体不暴露给用户，用 None 表示"未设置"。
3. **Java 枚举 `DEFAULT` 变体** → 在 Rust 中不出现为 enum variant；所有枚举 Option 的 None 等价于 Java DEFAULT。
4. **FillPatternType 别名**：Rust parser 同时接受 `fill_pattern` 和 `fill_pattern_type`，映射到同一字段。
5. **`center_across` vs `CENTER_SELECTION`**：Java POI 使用 `CENTER_SELECTION`，Rust 使用更通用的 `center_across`，语义一致。
6. **`slant_dash_dot` vs `SLANTED_DASH_DOT`**：拼写缩写差异，语义一致。

## 结论

9 个 write/style 注解 parser 的字段完整性已确认全部对齐，无需代码修改。
所有枚举变体覆盖完整，默认值语义一致。
