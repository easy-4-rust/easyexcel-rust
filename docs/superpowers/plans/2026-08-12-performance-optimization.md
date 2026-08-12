# 事件读追上 Java 吞吐 — 可执行任务清单

本文件是 easyexcel-rust【事件读追上 Java 吞吐】工作流的 WBS。每项任务都带
文件:行号证据、具体动作、可机器校验的验收标准和估算工作量。性能目标分解：

```
当前 205,551 rows/s（中位数；204,877 / 208,307 / 205,551，HOTSPOTS.md:22）
  │
  │  T1 BigDecimal 惰性化 + Converter from_f64 快路径
  ├─▶ 预期 ~260–290K rows/s（消除每个 numeric cell 的 BigDecimal 构造）
  │
  │  T2 行级 HashMap/HashSet scratch 复用
  ├─▶ 预期 ~285–315K rows/s（消除每行 4 个 HashMap 分配/rehash）
  │
  │  T3 typed scalar dispatch 快路径
  ├─▶ 预期 ~300–330K rows/s（no-extra/no-formula 直接走 typed emit）
  │
  │  T4 显式并发 Listener 有界管线（可选，单 worker 不依赖）
  ├─▶ 预期单 worker 不变；2/4 worker 需满足 internal_parallel_map ≥1.20 加速
  │
  │  T5/T6 benchmark 基线入库 + 跨运行时对比落地
  └─▶ 让 307K+ 目标可机器校验（cross_runtime gate min_median_ratio 1.00）
```

> Java 历史基线 307K–343K rows/s（HOTSPOTS.md:24）。最终单 worker 验收线定为
> **≥ 307,000 rows/s**（Linux release-ubuntu-x64 中位数），cross_runtime gate
> `rust/java ≥ 1.00`。

---

## 关键事实与证据（不要重复调研）

1. 当前事件读 205,551 rows/s 中位数，证据 `benchmarks/profiles/HOTSPOTS.md:22`。
2. **重要更正**：`retain_decimal_values` 在 typed 非 BigDecimal schema + 标准
   converter 路径下**已经是 false**（`read_xlsx.rs:51,107-116` 调
   `requires_decimal_metadata`，BenchmarkRow.score 是 `f64`，`schema()` 非空且
   无 `BigDecimal` 字段 → 返回 false）。因此 parser 端 BigDecimal（
   `xlsx_cell_event_reader.rs:241-248`）在 benchmark 里其实已被跳过。
3. **真正的 BigDecimal 热点在 Converter 层**：`read_number` 收到
   `CellValue::Float(value)` 时调 `T::from_f64(value)`（
   `number_support.rs:54`），而 `f64` 用的默认 `JavaNumber::from_f64` 会构造
   `BigDecimal::from_str(&value.to_string())`（`number_support.rs:30-34`）。
   `current-after-cell-scratch/xlsx-event-read.sample.txt` 的 call graph 证实：
   `from_str_radix` + `num_bigint` 在 `read_number` 下方（行 188-191）。
   这才是每个 numeric cell（BenchmarkRow.score）触发的逐格构造。
4. Checksum 约束：写侧 24,918,576 bytes →
   `df7966ddec70e23c9df5f8890d6c512c6ea1883d30f5283ac8d09d483f876c95`
   （HOTSPOTS.md:16）。读侧校验由 `compare_results.py` 跨实现比对
   （`compare_results.py:1311-1328`），任何优化必须保持 Rust/Java checksum 一致。
5. benchmark 基线目录只有 `README.md` + `baseline.schema.json`，无实际
   `.json` 基线（`benchmarks/baselines/`）。
6. cross_runtime gate 已在 spec 定义（`benchmark-suite-v1.json:86-93`），但
   `validate_release_inputs` 要求 Java 精确 tag `v4.0.3`（`run_matrix.py:720`），
   需要完整 release 流程才能跑通。

---

# WBS 任务清单

## T1 — BigDecimal 惰性化与 Converter 数值快路径（P0）

### T1.1 Converter `JavaNumber::from_f64` 增加 f64 直通快路径（P0，最高收益）

- **涉及文件**
  - `crates/easyexcel/src/converters/number_support.rs:30-34`（`from_f64` 默认实现）
  - `crates/easyexcel/src/converters/number_support.rs:250-258`（`f64::from_decimal`）
- **当前状态 + 证据**
  - 每个非整数 f64 单元格走 `CellValue::Float(v)` → `read_number` →
    `T::from_f64(v)` → 默认实现 `BigDecimal::from_str(&v.to_string())`
    （`number_support.rs:30-34`）→ `from_decimal` 再 `to_f64()` 回到 f64，一次
    f64→String→BigDecimal→f64 全程无意义往返。
  - 采样证据：`benchmarks/profiles/current-after-cell-scratch/xlsx-event-read.sample.txt`
    call graph 第 188-191 行 `BigDecimal::from_str_radix` + `num_bigint` BigUint
    allocate + `xzm_free`，全部挂在 `read_number` 下。
  - BenchmarkRow.score 是 `f64`（`benchmarks/rust-runner/src/benchmark_row.rs:15`），
    1,000,000 行每行触发一次 → 这是当前最大单一热点。
- **具体动作**
  1. 在 `impl JavaNumber for f64`（`number_support.rs:250`）重写 `from_f64`：
     ```rust
     fn from_f64(value: f64) -> Result<Self, ExcelError> { Ok(value) }
     ```
     并对 `f32`（`number_support.rs` 的 `impl JavaNumber for f32`，约 175 行）做同样直通。
  2. 保留 `from_decimal`/`to_decimal` 走 BigDecimal 的语义（write 路径与动态读仍需要）。
  3. 仅当 `value.is_finite()` 时直通；`NaN/Inf` 由 `read_number` 调用方已保证
     （`xlsx_cell_event_reader.rs:203` 拒绝 non-finite），但 `from_f64` 仍应防御性
     记录 `non_finite()`（已有实现，约 233-247 行）。
- **验收标准**
  - `cargo test -p easyexcel converters::number_support` 全绿（含既有
    `from_f64`/`from_decimal` 等价性测试）。
  - 读侧 checksum 不变：跑 `xlsx-event-read` scenario，checksum 与 Java runner 一致
    （`compare_results.py` 跨实现 checksum 集合大小为 1）。
  - macOS 短测中位吞吐 ≥ 245,000 rows/s（从 205K 提升 ≥19%）。
- **估算工作量**：3 小时（含测试补充）
- **依赖**：无
- **优先级**：P0

### T1.2 Parser 端 `retain_decimal_values` 默认值评审（P1）

- **涉及文件**
  - `crates/easyexcel-xlsx/src/xlsx/event_reader/readseek_to_read_comments/xlsx_display_options.rs:14,28`
  - `crates/easyexcel/src/read/read_xlsx.rs:51,107-116`（`requires_decimal_metadata`）
- **当前状态 + 证据**
  - `xlsx_display_options.rs:28` 默认 `retain_decimal_values: true`，注释
    （`:11-14`）说"静态 scalar model 可关掉避免逐格构造"。
  - 但 `read_xlsx.rs:51,107-116` 已经在 typed 非 BigDecimal + 标准 converter 路径
    上把它设成 false，所以 benchmark 里 parser 端 BigDecimal 已被跳过。
  - 真正风险：直接用 `XlsxCellEventReader` / `XlsxEventMetadata::cells`（
    `xlsx_event_metadata.rs:87-105`）的低层调用方仍拿到默认 true。
- **具体动作**
  1. **不要**改默认值（会破坏低层 API 语义与动态读 Java 对齐）。改为在
     `XlsxDisplayOptions::default()` 的 doc 注释里明确：高层 `EasyExcel::read`
     会按 schema 自动覆盖此值，低层 API 用户按需显式传 false。
  2. 把 `xlsx_cell_event_reader.rs:241-248` 的 `.then(|| number.to_string().parse::<BigDecimal>().ok())`
     加 `#[cold]` 提示或在 `retain_decimal_values` 为 true 时仍优先用
     `excel_display_number` 已有的 IEEE754 字符串表示，避免二次 `to_string()`。
  3. 仅作文档与微调；benchmark 不依赖此步。
- **验收标准**
  - `cargo doc -p easyexcel-xlsx` 无警告，注释准确。
  - `cargo test -p easyexcel-xlsx` 全绿。
- **估算工作量**：2 小时
- **依赖**：无
- **优先级**：P1

### T1.3 Converter 兼容性审计：动态读 / 自定义 Converter / write fill（P1）

- **涉及文件**
  - `crates/easyexcel/src/converters/number_support.rs`（`read_number`/`write_number`）
  - `crates/easyexcel/src/converters/string/string_number_converter.rs:19`（String 数值列用 `decimal_value`）
  - `crates/easyexcel/src/read/xlsx_rows/xlsx_display_cell.rs:7`（`decimal_value: Option<BigDecimal>`）
  - `crates/easyexcel/src/metadata/data/row_data.rs:30,182-186,211`（`decimal_values` HashMap + `decimal_value()`）
  - `crates/easyexcel/src/write/executor/excel_write_fill_executor.rs`（write fill，经 grep 确认存在）
  - `crates/easyexcel-derive/src/expand/conversion/read.rs:25,44,57`（derive 默认/自定义 converter 都读 `row.decimal_value(column)`）
- **当前状态 + 证据**
  - 动态读（`schema().is_empty()`）或自定义 converter 时
    `requires_decimal_metadata` 返回 true（`read_xlsx.rs:109-111`），parser 仍构造
    BigDecimal；derive 生成的自定义 converter 代码（`read.rs:24-28,43-48,56-61`）
    显式传 `row.decimal_value(column)`。
  - T1.1 只改 `from_f64` 默认实现；`from_decimal`（接收真正的 BigDecimal）与
    `to_decimal`（write 路径）不变，因此动态读 BigDecimal 字段、write fill、
    自定义 `BigDecimalNumberConverter` 全部不受影响。
- **具体动作**
  1. 列出所有 `impl JavaNumber` 的类型（f64/f32/i64/u64/BigDecimal/BigInt 等），
     审计每个 `from_f64` 是否能安全直通；除 BigDecimal/BigInt 外其余原语数值
     类型都应直通。
  2. 补一组单元测试：`read_number::<f64>` 在 `CellValue::Float(1.5)` 下不再分配
    BigDecimal（可用 `dhat`/`count_alloc` crate 的 allocation counter，或断言
    `from_f64(1.5) == Ok(1.5)` 且不 panic on non-finite guard）。
  3. 跑 `cargo test -p easyexcel` 全套，确认 write fill executor 与
    `BigDecimalNumberConverter` 测试不回归。
- **验收标准**
  - `cargo test -p easyexcel` 全绿。
  - 新增的 allocation-counter 测试断言 `read_number::<f64>` 在 Float 输入下
    allocation 次数为 0。
- **估算工作量**：4 小时
- **依赖**：T1.1
- **优先级**：P1

---

## T2 — 行级 HashMap/HashSet scratch 复用（P0）

### T2.1 `read_sheet` 行级容器改为 scratch 复用（P0）

- **涉及文件**
  - `crates/easyexcel/src/read/row_processing.rs:87-90`（4 个 `HashMap`/`HashSet` + `current_cells`）
  - `crates/easyexcel/src/read/row_consumer/source_row_metadata.rs:1-8`（`SourceRowMetadata` 拥有 4 个容器）
- **当前状态 + 证据**
  - `read_sheet` 在行切换时 `std::mem::take(&mut current_formulas)` 等
    （`row_processing.rs:103-109`）把容器所有权移进 `SourceRowMetadata`，再由
    `dispatch_row` → `process_row_with_metadata`（`row_consumer.rs:59-64`）解构消费，
    容器随之 drop —— 每行重新分配 4 个 HashMap + 1 个 Vec。
  - 采样证据：`current-after-display-column-plan` sample 显示
    `hashbrown::HashMap::insert` → `reserve_rehash` → `xzone_malloc_tiny` 占
    `XlsxCellEventReader::next_cell` 下方显著比例（sample 第 107 行起）。
  - typed 非 BigDecimal + 标准 converter 时这些 map 实际全空（dispatch_plan 已
    能跳 `present_columns`），但分配本身仍发生。
- **具体动作**
  1. 在 `read_sheet` 入口构造一组可复用的 scratch 容器，跨行 `clear()` 而非 drop：
     - `current_cells: Vec<CellValue>`（已存在，`:86`）
     - `current_formulas_scratch`、`current_display_values_scratch`、
       `current_decimal_values_scratch`、`current_present_columns_scratch`
  2. 引入 `SmallVec` 或 `tinyvec` 处理稀疏列（多数行列数 ≤ 16，避免 HashMap
     bucket 分配）。对 typed 强类型 schema，列位置已知且稠密，可直接用
     `Vec<Option<_>>` 以 column 为下标，消除 hashing。
  3. `dispatch_row` 改为接受 `&mut` 借用而非 `SourceRowMetadata` by-value，或在
     `process_row_with_metadata` 末尾把容器 `clear()` 后返还 `read_sheet`。
     需要改 `RowConsumer::process` 签名（`row_consumer.rs:27-36`）—— 评估能否
     保持 by-value 但用 `Result<SourceRowMetadata>` 归还（ ergonomic 折中）。
  4. 当 `dispatch_plan` 表明 typed 无需 formulas/display/decimal/present 时，
     完全不构造这些容器（lazy 初始化）。
- **验收标准**
  - `cargo test -p easyexcel` 全绿（含 `row_consumer.rs:104-310` 的契约测试）。
  - 读 checksum 不变。
  - macOS 短测中位吞吐 ≥ 270,000 rows/s。
  - 用 `cargo run --release --features dhat` 或 `count_alloc` 测得每行 HashMap
    分配次数从 ≥4 降到 0（稳态）。
- **估算工作量**：8 小时
- **依赖**：T1.1（先消除 BigDecimal 再测真实 allocator 收益）
- **优先级**：P0

### T2.2 `SourceRowMetadata` 结构与 `RowData::from_stream_parts` 适配（P1）

- **涉及文件**
  - `crates/easyexcel/src/read/row_consumer.rs:46-102`（`process_row_with_metadata` → `RowData::from_stream_parts`）
  - `crates/easyexcel/src/metadata/data/row_data.rs:30,86,182-211`（`RowData.decimal_values: HashMap`）
- **当前状态 + 证据**
  - `RowData::from_stream_parts` 接收 by-value `HashMap`（`row_data.rs:86`），
    T2.1 若改成 scratch 借用需要同步调整签名或新增一个 `from_stream_parts_borrowed`。
- **具体动作**
  1. 为 `RowData` 增加一个接受 `&HashMap` 或 `SmallVec<[(usize,V); N]>` 的构造变体，
     供 typed 快路径使用；保留原 by-value 版本给动态读。
  2. 动态读（`schema().is_empty()`）路径保持现状（需要完整 HashMap 语义）。
- **验收标准**
  - `cargo test -p easyexcel` 全绿。
  - 动态读（`ReadListener<RowData>`）checksum 与 typed 读一致。
- **估算工作量**：4 小时
- **依赖**：T2.1
- **优先级**：P1

---

## T3 — typed scalar dispatch 快路径（P1）

### T3.1 no-extra/no-formula/no-decimal 专用 dispatch 代码路径（P1）

- **涉及文件**
  - `crates/easyexcel/src/read/read_dispatch_plan.rs:1-27`（当前只判 `retain_present_columns`）
  - `crates/easyexcel/src/read/row_consumer.rs:20-41`（`RowConsumer` trait）
  - `crates/easyexcel/src/read/row_consumer/typed_row_consumer.rs:1-40`
  - `crates/easyexcel/src/read/row_processing.rs:75-195`（`read_sheet` 主循环）
  - `crates/easyexcel/src/read/read_xlsx.rs:107-134`（`requires_decimal_metadata`/`required_display_columns`）
- **当前状态 + 证据**
  - `ReadDispatchPlan` 只携带 `retain_present_columns` 一个布尔（`read_dispatch_plan.rs:9-11`）。
  - `read_sheet` 即使 typed schema 无 formula/extra/decimal，仍每行走完整
    `dispatch_row` → `process_row_with_metadata` → 构造 `RowData` →
    `T::from_row_with_converters`（`row_consumer.rs:84-101`）的通用所有权转换。
  - 采样：`dispatch_row` + `TypedRowConsumer::process` 在 cell-scratch sample 占
    ~704+191 samples（行 704-191），其中 `from_row_with_converters` 191 样本里
    有相当比例是 `RowData` 装配 + converter 调度开销。
- **具体动作**
  1. 扩展 `ReadDispatchPlan`，在 `compile` 时（`read_dispatch_plan.rs:16-20`）多收集：
     - `retain_formulas: bool`（schema 无 formula 字段且无 extras → false）
     - `retain_display_values: bool`（`required_display_columns` 为空 set → false）
     - `retain_decimal_values: bool`（= `requires_decimal_metadata` 结果）
     - `typed_scalar_fast_path: bool`（强类型 + 标准 converter + 无 extras）
  2. 在 `read_sheet` 主循环按 plan 分支：fast path 直接把 `current_cells` 交给
     一个轻量 `TypedRowConsumer::process_fast`，跳过 `SourceRowMetadata` 装配。
  3. 注意 `extras` 非空时（merge/hyperlink/comment）必须回退到完整路径
     （`row_processing.rs:188-193` 的 extras 分发不能跳过）。
- **验收标准**
  - `cargo test -p easyexcel` 全绿，含 `row_consumer.rs:104-310` 契约测试。
  - 读 checksum 不变。
  - macOS 短测中位吞吐 ≥ 295,000 rows/s。
  - extras 非空场景（如 roundtrip / 含批注）走完整路径，行为不变（新增一个
    带 comment 的 fixture 测试断言 extra 仍被分发）。
- **估算工作量**：10 小时
- **依赖**：T2.1
- **优先级**：P1

### T3.2 derive 标注字段类型直读快路径（P2）

- **涉及文件**
  - `crates/easyexcel-derive/src/expand/conversion/read.rs:1-62`（`field_read_conversion` / `field_registered_read_conversion`）
  - `crates/easyexcel/src/converters/converter_registry.rs:290-296`（`is_standard_read_only`）
- **当前状态 + 证据**
  - 当 `converters.is_standard_read_only()` 为 true 时，derive 已能走
    `convert_to_rust_data` 快路径（`read.rs:41-50`），但仍构造完整
    `ReadConverterContext::with_cell_metadata`（含 `decimal_value` 查询）。
  - `FromExcelCell`（`read.rs:15-17`）路径不走 converter registry，已经较轻。
- **具体动作**
  1. 在 derive 里为纯原语字段（i64/f64/String/bool/NaiveDate 且无自定义
     converter）直接展开 `CellValue` 模式匹配，绕过 `ReadConverterContext` 装配。
  2. 保持自定义 converter 与非标准 registry 走原路径。
- **验收标准**
  - `cargo test -p easyexcel-derive` 全绿。
  - 读 checksum 不变。
- **估算工作量**：6 小时
- **依赖**：T3.1
- **优先级**：P2

---

## T4 — 显式并发 Listener 有界管线（P2）

> 单 worker 目标 307K+ 不依赖本节。本节服务于 release `internal_parallel_map`
> gate（`benchmark-suite-v1.json:27-35`，`min_median_speedup 1.20`）与
> `cross_runtime` 高并发 gate（`:89-92` `min_high_concurrency_median_ratio 0.90`）。

### T4.1 `ParallelMapReadListener` XML 解析并发化调研（P2，待确认）

- **涉及文件**
  - `crates/easyexcel/src/read/listener/parallel_map_read_listener.rs:1-302`
  - `benchmarks/rust-runner/src/operation.rs:109-130`（用法）
- **当前状态 + 证据**
  - 现有 `ParallelMapReadListener` 只并发 `mapper`，XML 解析与下游
    `invoke` 仍单线程有序（`parallel_map_read_listener.rs:34-38` 注释明确）。
  - HOTSPOTS.md:29 提出"显式并发 Listener 有界'解析—转换—有序提交'管线"，
    即让 XML 解析也并发。但 XLSX 是单 XML 流，按 sheet/row 分片解析需要
    预扫或随机访问，复杂度高。
- **具体动作**
  1. 评估两种方案：
     - A. 保持单 XML 解析，仅把"行→typed row"转换（converter + RowData 装配）
       放进 worker，listener.invoke 仍有序。这其实就是当前 ParallelMapReadListener
       的语义，只需让 BenchmarkRow 的 `apply_benchmark_map` 之外的真实转换受益。
     - B. 多 sheet 并发（每个 sheet 一个解析线程），适合 workbook > 1 sheet 场景；
       benchmark 单 sheet 不受益。
  2. 输出一份设计 note 决定是否值得做。**待确认**：是否接受方案 A 作为本任务范围。
- **验收标准**
  - 产出设计 note（不一定要实现）。
  - 若实现方案 A：`internal_parallel_map` 在 2 worker 下 median speedup ≥ 1.20
    （`benchmark-suite-v1.json:33`），peak RSS ≤ 67,108,864 bytes（`:34`）。
- **估算工作量**：调研 4 小时；实现 16 小时
- **依赖**：T1.1, T3.1
- **优先级**：P2

### T4.2 benchmark `internal_parallel_map` gate 本地验证（P2）

- **涉及文件**
  - `benchmarks/rust-runner/src/operation.rs:109-130`（parallel-map 入口）
  - `benchmarks/scripts/run_matrix.py:539-596`（`run_internal_parallel_map`）
  - `benchmarks/scripts/compare_results.py:1165-1213`（`enforce_internal_parallel_map_gate`）
- **当前状态 + 证据**
  - `internal_parallel_map` 只在 release profile 跑（`run_matrix.py:1033`），
    worker_counts `[1,2,4]`，queue_capacity_per_worker 64，work_factor 32
    （`benchmark-suite-v1.json:27-35`）。
- **具体动作**
  1. 在本地用 nightly profile（rows 100000）跑一次 `xlsx-event-read` +
    `--internal-map-work-factor 32 --internal-map-queue-capacity <64*workers>`，
    记录 1/2/4 worker 中位吞吐。
  2. 确认 2/4 worker speedup ≥ 1.20；若不达标，回到 T4.1 评估 mapper 并发粒度。
- **验收标准**
  - 本地 nightly 跑通，2 worker speedup ≥ 1.20。
- **估算工作量**：3 小时
- **依赖**：T4.1
- **优先级**：P2

---

## T5 — benchmark 基线入库（P1）

### T5.1 选定 Linux 固定环境并固化 7 样本基线（P1）

- **涉及文件**
  - `benchmarks/baselines/README.md:1-47`（基线流程）
  - `benchmarks/baselines/baseline.schema.json`（schema-v2）
  - `benchmarks/scripts/approve_benchmark_baseline.py`（批准脚本）
  - `benchmarks/scripts/run_matrix.py:962-1048`（`main`）
  - `benchmarks/spec/benchmark-suite-v1.json:18`（release profile：7 measurements）
- **当前状态 + 证据**
  - `benchmarks/baselines/` 只有 `README.md` + `baseline.schema.json`，无
    `nightly-ubuntu-x64.json` 或 `release-ubuntu-x64.json`
    （README:8-10 明确"未生成前 workflow 报 missing-baseline"）。
  - release profile 要求 rows `[1000000]`、temperatures `[cold,steady]`、
    warmups 3、measurements 7、duration_seconds 1800
    （`benchmark-suite-v1.json:18`）。
- **具体动作**
  1. 在固定 Linux x64 runner（与 release CI 同机型）跑：
     ```
     python3 benchmarks/scripts/run_matrix.py \
       --profile release --rust-bin <prebuilt> --java-bin java \
       --java-classpath <...> --java-repo <java v4.0.3> \
       --rust-repo . --artifact-manifest <...> \
       --output-dir /tmp/release-run
     ```
  2. 用 `prepare_release_artifacts.py` 生成 attestation（`run_matrix.py:710-791`
     的 `validate_release_inputs` 强制要求）。
  3. 用 `approve_benchmark_baseline.py` 把候选 report 固化为
     `benchmarks/baselines/release-ubuntu-x64.json`，需显式提供每个 JSONL
     evidence（README:24-33）。
- **验收标准**
  - `benchmarks/baselines/release-ubuntu-x64.json` 存在且通过
    `compare_results.py::validate_stable_baseline`（`compare_results.py:854-942`）。
  - 基线 `summaries` 包含全部 9 个 scenario 的中位吞吐。
  - 基线 source_git_shas 绑定 Rust 当前 HEAD + Java `v4.0.3`。
- **估算工作量**：6 小时（含环境调试）
- **依赖**：T1.1（先优化再固化，否则基线立即被超越）
- **优先级**：P1

### T5.2 nightly 基线与 regression gate 联调（P2）

- **涉及文件**
  - `benchmarks/scripts/compare_results.py:1468-1484`（baseline regression 比对）
  - `benchmarks/spec/benchmark-suite-v1.json:67-80`（gates：max CV 0.10、
    max median regression 0.10、max peak RSS regression 0.15）
- **当前状态 + 证据**
  - regression gate 已实现（`compare_results.py:1478-1484`），但缺 nightly 基线。
- **具体动作**
  1. 同 T5.1 流程，用 nightly profile（rows 100000、7 measurements）生成
     `benchmarks/baselines/nightly-ubuntu-x64.json`。
  2. 验证 `--require-baseline` 在 nightly CI 上能拒绝 >10% 回归。
- **验收标准**
  - `nightly-ubuntu-x64.json` 通过 schema 校验。
  - 人为制造 -15% 回归被 gate 拒绝（手动验证）。
- **估算工作量**：3 小时
- **依赖**：T5.1
- **优先级**：P2

---

## T6 — 跨运行时（Java vs Rust）对比落地（P1）

### T6.1 Java v4.0.3 tag runner 构建与 classpath 固化（P1）

- **涉及文件**
  - `benchmarks/java-runner/src/`（Java runner 源，被 `run_matrix.py:776-781` 校验）
  - `benchmarks/scripts/run_matrix.py:710-791`（`validate_release_inputs`）
  - `benchmarks/scripts/prepare_release_artifacts.py`（attestation）
- **当前状态 + 证据**
  - `validate_release_inputs`（`run_matrix.py:720`）要求 Java repo 精确 tag
    `v4.0.3`，classpath 首项必须是
    `<java-repo>/easyexcel-test/target/test-classes`（`:723-729`）。
  - `artifact_manifest` schema_version 3，含 Rust rustc 指纹与 Java classpath
    全量 SHA（`:737-791`）。
- **具体动作**
  1. clone easyexcel java 到固定路径，`git checkout v4.0.3`。
  2. 把 `benchmarks/java-runner/src` 的 runner 类编译进
     `easyexcel-test/target/test-classes`。
  3. 跑 `prepare_release_artifacts.py` 生成 `artifact-manifest.json`。
  4. 记录 Java 17 / G1 / Xms512m / Xmx4g（`benchmark-suite-v1.json:36-45`）。
- **验收标准**
  - `python3 benchmarks/scripts/run_matrix.py --profile release ...` 的
    `validate_release_inputs` 通过（`:710-734`）。
  - `artifact-manifest.json` schema_version=3，Rust rustc SHA 与 binary SHA 匹配。
- **估算工作量**：6 小时
- **依赖**：T5.1
- **优先级**：P1

### T6.2 cross_runtime gate 在 `[1,2,4,8,16]` worker 全量验证（P0，最终验收）

- **涉及文件**
  - `benchmarks/spec/benchmark-suite-v1.json:20,86-93`（concurrency、cross_runtime gate）
  - `benchmarks/scripts/compare_results.py:1386-1430`（cross_runtime ratio + gate）
  - `benchmarks/scripts/compare_results.py:260-311`（`summarize_concurrent_throughput`、`bootstrap_median_ratio`）
- **当前状态 + 证据**
  - cross_runtime gate 定义齐全：scenarios `[xlsx-stream-write, xlsx-event-read]`、
    worker_counts `[1,2,4,8,16]`、`min_median_ratio 1.00`、
    `min_confidence_lower_bound 0.95`、高并发 `[8,16]` 用
    `min_high_concurrency_median_ratio 0.90`（`:86-93`）。
  - 比对逻辑已实现（`compare_results.py:1391-1430`），bootstrap 10000 次确定性
    seed（`:291-311`）。
  - 但因 T5/T6.1 未落地，release 流程从未跑通。
- **具体动作**
  1. 完整 release run（T5.1 + T6.1 就绪后），确保
     `xlsx-stream-write` 与 `xlsx-event-read` 在 `[1,2,4,8,16]` worker 都有
     7×worker_count 个样本（`compare_results.py:328-359` matrix 完整性）。
  2. 检查 `compare_results.py` 输出的 `cross_runtime_ratios`，确认
     `xlsx-event-read` 在 worker=1 时 `median_ratio ≥ 1.00` 且
     `confidence_lower_bound ≥ 0.95`。
- **验收标准（最终）**
  - `python3 benchmarks/scripts/compare_results.py <raw> --spec ... --profile release --baseline benchmarks/baselines/release-ubuntu-x64.json --expected-java-git-sha <v4.0.3 sha> --expected-rust-git-sha <rust sha>` 退出码 0。
  - `xlsx-event-read` worker=1 cross_runtime `median_ratio ≥ 1.00`
    （即 Rust 中位 ≥ Java 中位，对应 ~307K+ rows/s）。
  - 读 checksum 跨 Rust/Java/neutral 三 origin 一致
    （`compare_results.py:1311-1328`）。
- **估算工作量**：4 小时（不含环境等待）
- **依赖**：T5.1, T6.1, T1.1, T2.1, T3.1
- **优先级**：P0（最终里程碑）

---

## 总览：任务依赖与优先级

| 编号 | 标题 | 优先级 | 估算(h) | 依赖 | 目标吞吐 |
|------|------|--------|---------|------|----------|
| T1.1 | Converter `from_f64` f64 直通快路径 | P0 | 3 | — | 205K→245K+ |
| T1.2 | Parser `retain_decimal_values` 默认值评审 | P1 | 2 | — | 文档 |
| T1.3 | Converter 兼容性审计（动态读/write fill） | P1 | 4 | T1.1 | 不回归 |
| T2.1 | `read_sheet` 行级容器 scratch 复用 | P0 | 8 | T1.1 | →270K+ |
| T2.2 | `SourceRowMetadata`/`RowData` 适配 | P1 | 4 | T2.1 | 不回归 |
| T3.1 | no-extra/no-formula/no-decimal dispatch 快路径 | P1 | 10 | T2.1 | →295K+ |
| T3.2 | derive 原语字段直读快路径 | P2 | 6 | T3.1 | 微增 |
| T4.1 | `ParallelMapReadListener` 解析并发调研 | P2 | 4+16 | T1.1,T3.1 | 待确认 |
| T4.2 | `internal_parallel_map` gate 本地验证 | P2 | 3 | T4.1 | ≥1.20x |
| T5.1 | Linux 固定环境 7 样本 release 基线入库 | P1 | 6 | T1.1 | 可校验 |
| T5.2 | nightly 基线与 regression gate 联调 | P2 | 3 | T5.1 | 可校验 |
| T6.1 | Java v4.0.3 runner 构建与 classpath 固化 | P1 | 6 | T5.1 | 可跑 |
| T6.2 | cross_runtime gate 全量验证（最终验收） | P0 | 4 | T5.1,T6.1,T1.1,T2.1,T3.1 | ≥1.00 |

合计 13 个任务项，总估算约 73 小时（含 T4.1 实现 16h）。

---

## 验证用的通用命令速查

```shell
# 单元测试
cargo test -p easyexcel-xlsx
cargo test -p easyexcel
cargo test -p easyexcel-derive

# 本地 macOS 短测（不替代 Linux release 基线）
cargo build --release -p easyexcel-benchmark-runner
./target/release/easyexcel-benchmark-runner \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --scenario xlsx-event-read --rows 1000000 --workers 1 \
  --temperature steady --warmups 3 \
  --input <fixture.xlsx>

# 读 checksum 比对（Rust vs Java，需双方 runner 就绪）
python3 benchmarks/scripts/compare_results.py \
  <rust-raw.jsonl> <java-raw.jsonl> \
  --spec benchmarks/spec/benchmark-suite-v1.json \
  --profile nightly

# 完整 release（需 Linux 固定环境 + Java v4.0.3 + artifact manifest）
python3 benchmarks/scripts/run_matrix.py --profile release \
  --rust-bin <bin> --java-bin java --java-classpath <cp> \
  --java-repo <java v4.0.3 repo> --rust-repo . \
  --artifact-manifest <artifact.json> --output-dir <out>
```

## 待确认事项

1. **T4.1**：是否接受"仅并发行转换、XML 解析仍单线程"（方案 A）作为并发管线
   任务范围，还是必须做多 sheet/多流 XML 解析并发（方案 B）？方案 B 对 benchmark
   单 sheet 场景无收益。
2. **T2.1**：`RowConsumer::process` 签名变更（by-value → 借用/归还）会影响 trait
   对象 ABI，是否允许？若不允许，则用"消费后归还 `SourceRowMetadata`"的
   `Result<SourceRowMetadata>` 折中。
3. **T5.1 环境**：Linux 固定 runner 机型/CPU/内存规格待运维确认；基线一旦固化，
   后续优化必须在该机型复测才能进 `release-ubuntu-x64.json`。
