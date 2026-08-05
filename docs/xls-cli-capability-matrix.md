# xls-cli 能力矩阵

> 运行时权威：`xls capabilities --json`
>
> `supported` 表示当前实现可调用；`unsupported` 表示协议保留但返回 `UNSUPPORTED_COMMAND`。

## 命令

| 分组 | 命令 | 状态 | 实现/边界 |
| --- | --- | --- | --- |
| 检查 | `info` | supported | sheets、dimensions、公式/merge/table 数量 |
| 检查 | `get` | supported | A1 范围，JSON/CSV/TSV/Markdown/HTML |
| 检查 | `head` / `tail` | supported | 指定 sheet 与行数 |
| 检查 | `grep` / `profile` | unsupported | 不降级 |
| 编辑 | `set` / `clear` / `fill` | supported | 新输出或显式 `--force` |
| 编辑 | `insert-row` / `delete-row` | supported | 0 基索引；merge/table 同步调整 |
| 编辑 | `insert-col` / `delete-col` | supported | 0 基索引；中间删 table 列名是已知限制 |
| 编辑 | `copy` / `move` | unsupported | 不降级 |
| 工作簿 | `new` / `add-sheet` / `delete-sheet` / `rename-sheet` | supported | 校验 Excel sheet 命名规则 |
| 工作簿 | `append` | unsupported | 不降级 |
| 数据 | `query` | supported | SELECT、WHERE、GROUP BY、JOIN、ORDER BY、LIMIT |
| 数据 | `filter` / `sort` / `dedup` / `join` / `pivot` / `diff` | unsupported | 不降级 |
| 格式 | `format` / `style` / `autofit` | unsupported | 不降级 |
| 交换 | `convert` | supported | XLS/XLSX/CSV 由输出扩展名决定 |
| 交换 | `import` | supported | Markdown/HTML/JSON → XLS/XLSX/XLS/CSV |
| 交换 | `export` | supported | Markdown/HTML/JSON/CSV/TSV |
| 交换 | `batch` | unsupported | 不降级 |
| 元数据/公式 | `recalc` | supported | 全工作簿公式缓存重算，报告 circular |
| 元数据/公式 | `name` / `table` / `eval` | unsupported | 模型/公式底层已有部分能力，命令尚未开放 |
| 协议 | `capabilities` / `schema` | supported | JSON protocol 1.0 |

## 格式与模式

| 能力 | XLS | XLSX | CSV | Markdown | HTML | JSON |
| --- | --- | --- | --- | --- | --- | --- |
| Workbook read | supported | supported | supported | import | import | import |
| Event read | EasyExcel facade | supported (`xlsx::stream`) | EasyExcel facade | — | — | — |
| Generate | supported | supported | supported | export | export | export |
| RoundTrip | partial/tested | partial/tested | value roundtrip | — | — | — |
| Password read | unsupported legacy RC4 in new foundation | supported MS-OFFCRYPTO | — | — | — | — |
| Multi-table import | — | — | single sheet | supported | supported | supported |
| Merge import | — | — | — | parser has no merge syntax | rowspan/colspan | schema carries merges on export only |

## 特征测试证据

| Crate | 本阶段定向结果 | 重点覆盖 |
| --- | ---: | --- |
| `easyexcel-xls` | 13 passed | BIFF8 records、SST、multisheet、format/date、container roundtrip |
| `easyexcel-xlsx` | 19 passed | values/formulas、styles、defined names、opaque parts、tables、stream、password detection |
| `easyexcel-csv` | 7 passed | BOM、delimiter、encoding fallback、type inference、ragged rows、roundtrip |
| `easyexcel-tabular` | 5 passed | multi-table Markdown/HTML/JSON、HTML merge、sheet name safety |
| `xls-cli` library | 15 passed | query engine + capabilities + unsupported + dry-run + overwrite + Markdown E2E |

这些计数是定向 crate 测试，不等价于整个 workspace 或发布平台已经完成验证。
