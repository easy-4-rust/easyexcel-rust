# xls-cli 能力矩阵

> 运行时权威：`xls capabilities --json`
>
> `supported` 表示结构化协议与终端路径均可调用；`partial` 表示终端命令可用、结构化 JSON 请求仍返回 `UNSUPPORTED_COMMAND`。

## 命令

| 分组 | 命令 | 状态 | 实现/边界 |
| --- | --- | --- | --- |
| 检查 | `info` | supported | sheets、dimensions、公式/merge/table 数量 |
| 检查 | `get` | supported | A1 范围，JSON/CSV/TSV/Markdown/HTML |
| 检查 | `head` / `tail` | supported | 指定 sheet 与行数 |
| 检查 | `grep` / `profile` | partial | 终端可用；结构化请求不降级 |
| 编辑 | `set` / `clear` / `fill` | supported | 新输出或显式 `--force` |
| 编辑 | `insert-row` / `delete-row` | supported | 0 基索引；merge/table 同步调整 |
| 编辑 | `insert-col` / `delete-col` | supported | 0 基索引；中间删 table 列名是已知限制 |
| 编辑 | `copy` / `move` | partial | 终端可用；结构化请求不降级 |
| 工作簿 | `new` / `add-sheet` / `delete-sheet` / `rename-sheet` | supported | 校验 Excel sheet 命名规则 |
| 工作簿 | `append` | partial | 终端可用；结构化请求不降级 |
| 数据 | `query` | supported | SELECT、WHERE、GROUP BY、JOIN、ORDER BY、LIMIT |
| 数据 | `filter` / `sort` / `dedup` / `join` / `pivot` / `diff` | partial | 终端可用；结构化请求不降级 |
| 格式 | `format` / `style` / `autofit` | partial | 终端可用；结构化请求不降级 |
| 交换 | `convert` | supported | XLS/XLSX/CSV 由输出扩展名决定 |
| 交换 | `import` | supported | Markdown/HTML/JSON → XLS/XLSX/XLS/CSV |
| 交换 | `export` | supported | Markdown/HTML/JSON/CSV/TSV |
| 交换 | `batch` | partial | 终端可用；结构化请求不降级 |
| 元数据/公式 | `recalc` | supported | 全工作簿公式缓存重算，报告 circular |
| 元数据/公式 | `name` / `table` / `eval` | partial | 终端可用；结构化请求仍明确 unsupported |
| 协议 | `capabilities` / `schema` | supported | JSON protocol 1.0 |

## 格式与模式

| 能力 | XLS | XLSX | CSV | Markdown | HTML | JSON |
| --- | --- | --- | --- | --- | --- | --- |
| Workbook read | supported | supported | supported | import | import | import |
| Event read | 不支持，明确返回错误 | supported (`xlsx::stream`) | supported (`CsvRowSource`) | — | — | — |
| Generate | supported | supported | supported | export | export | export |
| RoundTrip | partial/tested | partial/tested | value roundtrip | — | — | — |
| Password read | unsupported legacy RC4 in new foundation | supported MS-OFFCRYPTO | — | — | — | — |
| Multi-table import | — | — | 必须选择单表 | supported | supported | supported |
| Merge import | — | — | — | parser has no merge syntax | rowspan/colspan | schema carries merges on export only |

## 特征测试证据

| Crate | 本阶段定向结果 | 重点覆盖 |
| --- | ---: | --- |
| `easyexcel-xls` | 13 passed | BIFF8 records、SST、multisheet、format/date、container roundtrip |
| `easyexcel-xlsx` | 19 passed | values/formulas、styles、defined names、opaque parts、tables、stream、password detection |
| `easyexcel-csv` | 12 unit + 2 integration passed | BOM、delimiter、encoding、type inference、roundtrip、首行前不读完整文件 |
| `easyexcel-markdown` | 9 passed | GFM 多表、转义、保守推断、公式、四种 merge 策略、资源限制与 warning |
| `easyexcel-tabular` | 3 passed | 静态 HTML/JSON；Markdown 分派委托独立投影层 |
| `easyexcel` | 1396 unit + 7 integration passed | 原门面回归与 XLS/XLSX/CSV ↔ Markdown、Event/Workbook 对照 |
| `xls-cli` | 104 unit + 3 process protocol passed | capabilities、schema、dry-run、overwrite、JSON stdout 与 Markdown E2E |

本阶段同时通过 `cargo test --workspace --all-features` 与 workspace `clippy -D warnings`；跨平台 npm 原生包仍属于独立发布流水线验收。
