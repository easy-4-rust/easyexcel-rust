# Fill Demo (CLI)

Demonstrates template filling: writes a template XLSX with placeholder cells,
then calls `EasyExcel::fill_template` to produce a filled output file. This
mirrors the Java `easyexcel-demo` fill module's main entry point.

## Quick start

```bash
cargo run -p easyexcel-demo-fill
```

No arguments are needed; the template and output paths are fixed under
`target/`.

## What it does

1. Creates `target/demo-fill-template.xlsx` with two columns:
   - Column 0 ("姓名"): placeholder text `{name}`
   - Column 1 ("分数"): placeholder text `{score}`
2. Builds a `TemplateData` map: `{name} -> "张三"`, `{score} -> 98.5`.
3. Calls `EasyExcel::fill_template(&template, &output, &data)` to produce
   `target/demo-fill-output.xlsx` with the placeholders replaced.
4. Prints both paths to stdout.

## Data model

### Template seed row

| Column | Name | Placeholder | Index |
|---|---|---|---:|
| 0 | 姓名 | `{name}` | 0 |
| 1 | 分数 | `{score}` | 1 |

### Fill data

| Key | Value |
|---|---|
| `name` | 张三 |
| `score` | 98.5 |

## Input / Output

- **Input**: None (template is generated in-memory, then filled).
- **Output**:
  - `target/demo-fill-template.xlsx` -- the intermediate template.
  - `target/demo-fill-output.xlsx` -- the final filled workbook.

## Correspondence with Java demo

| Java class | Rust equivalent |
|---|---|
| `com.alibaba.easyexcel.demo.fill.FillDemo` | `examples/fill/src/main.rs` |
| `EasyExcel.fill(template, output, data)` | `EasyExcel::fill_template(&template, &output, &data)` |

The template approach follows the same pattern as the Java `simple.xlsx` fill
example: write placeholder text into cells, then substitute values from a map.

## Associated test

The integration test `tests/spawn_binary.rs` verifies the demo binary:

1. Spawns `easyexcel-demo-fill` as a subprocess in a temporary working
   directory.
2. Asserts exit code 0, stdout contains "模板:" and "输出:".
3. Verifies the filled output file exists and starts with `PK` bytes (valid
   XLSX).

Run it with:

```bash
cargo test -p easyexcel-demo-fill
```
