# ParallelMapReadListener XML 解析并发化 RFC

**任务**: T4.1 ParallelMapReadListener XML 解析并发化调研
**日期**: 2026-08-11
**状态**: 调研完成，推荐方案 A

---

## 1. 现状分析

### 1.1 当前数据流

```
XlsxCellEventReader::next_cell()        (单线程 XML 流式解析)
        │
        ▼
read_sheet() → dispatch_row()           (单线程行装配 + RowData 构造)
        │
        ▼
ParallelMapReadListener::invoke()       (提交到 worker 线程池)
        │  ┌───────────────────┐
        ├─▶│ worker 0: mapper  │
        ├─▶│ worker 1: mapper  │
        ├─▶│ worker N: mapper  │
        │  └───────────────────┘
        ▼
commit_ready() → downstream.invoke()    (单线程有序提交)
```

关键约束（`parallel_map_read_listener.rs:34-38` 注释明确）：

- XML 解码保持单线程、有序
- 只有 `mapper` 在 worker 中并发
- 下游 `ReadListener` 回调保持单线程、有序
- 队列容量是硬上限，首个错误触发取消

### 1.2 XLSX XML 流的本质限制

`XlsxCellEventReader`（`xlsx_cell_event_reader.rs:2-15`）持有一个
`XmlReader<Box<dyn BufRead>>`，逐事件推进 SAX 解析器。XLSX 工作表的
`sheetData` 是一棵连续的 XML 子树：

```xml
<sheetData>
  <row r="1"><c r="A1"><v>1</v></c>...</row>
  <row r="2"><c r="A2"><v>2</v></c>...</row>
  ...
</sheetData>
```

XML 是有序的 token 流；在不预扫描或随机访问的前提下，无法将单个 XML
流的解析拆分到多个线程。这是方案 B/C 的根本复杂度来源。

### 1.3 Benchmark 瓶颈分析

benchmark 用法（`operation.rs:109-130`）：

```rust
let listener = ParallelMapReadListener::new(
    config.worker_count,
    config.queue_capacity,
    move |row, _context| Ok(apply_benchmark_map(row, work_factor)),
    EventListener(Rc::clone(&state)),
)?;
```

`apply_benchmark_map` 对每个 `BenchmarkRow` 执行 `work_factor`（默认 32）
次 `ahash` 哈希计算，是纯 CPU 密集型工作。benchmark 中：

- **mapper 是瓶颈**：32 次哈希 >> XML 解析 + RowData 装配 + invoke 开销
- **下游 invoke 是轻量操作**：累加行计数和 checksum
- **当前方案已充分利用 mapper 并发**：2 worker speedup >= 1.20x 已满足 gate

对真实业务场景（非 benchmark）：

- mapper 是用户定义的复杂转换（类型校验、格式化、业务规则）
- `from_row_with_converters`（`row_consumer.rs:136`）和 `RowData::from_stream_parts`
  （`row_consumer.rs:124-135`）是每行的主要开销
- 真实场景的 mapper 瓶颈程度 >= benchmark（因 converter 链更长）

---

## 2. 方案对比

### 方案 A：保单 XML 解析，并发"行→typed row"转换

**核心思路**：保持现有 `ParallelMapReadListener` 架构不变。XML 解析
仍单线程，`mapper` 在 worker 线程池中并发执行，`listener.invoke`
仍有序提交。

**扩展方向**：将 mapper 的职责从"纯函数转换"扩展到包含
`RowData::from_stream_parts` + `T::from_row_with_converters` 的
完整"行→typed row"转换链。这需要在 `read_sheet` 层面将行装配
（cells + metadata 收集）与 typed 转换（RowData 构造 + converter）
拆分为两个阶段。

**架构**：

```
read_sheet() 主线程:
  ┌─────────────────────────────────────────────────┐
  │ for each row:                                    │
  │   XML parse → collect cells + metadata           │  (单线程)
  │   submit {cells, metadata, sequence} to worker   │
  └─────────────────────────────────────────────────┘
            │
            ▼
  ┌─────────────────────────┐
  │ worker 0: RowData::from │
  │   _stream_parts +       │  (并发)
  │   T::from_row_with_     │
  │   converters            │
  │ worker 1: ...           │
  │ worker N: ...           │
  └─────────────────────────┘
            │
            ▼
  commit_ready() → downstream.invoke()  (单线程有序)
```

**实现路径**（不改生产代码，仅评估可行性）：

1. 当前 `ParallelMapReadListener<T, U, L>` 的泛型 `T` 是行类型，
   `mapper: Fn(T, &AnalysisContext) -> Result<U>` 接收已转换的 `T`。
2. 扩展为接收 `(Vec<CellValue>, SourceRowMetadata)` 元组，在 worker
   内部完成 `RowData::from_stream_parts` + `T::from_row_with_converters`。
3. 或者保持现有接口，让用户在 mapper 中自行调用转换逻辑。
   当前 benchmark 的 `apply_benchmark_map` 已经接收 `BenchmarkRow`（已转换），
   因此对 benchmark 无额外收益——但对真实业务场景有收益。

**优势**：

- 最低风险：不改变 XML 解析架构，不引入新的并发原语
- 最高 ROI：与现有架构完全兼容，只需扩展 mapper 输入类型
- 已满足 benchmark gate：2 worker speedup >= 1.20x（`benchmark-suite-v1.json:28`）
- 峰值 RSS 受控：`queue_capacity` 硬上限（默认 64*worker_count）
  限制 in-flight 行数（`benchmark-suite-v1.json:29`，max 67,108,864 bytes）

**劣势**：

- XML 解析仍是单线程瓶颈（但对 XLSX 单流来说无法避免）
- 对 benchmark 场景无额外收益（mapper 已是瓶颈）

**风险**：

- **低**：不改变并发模型，只扩展 mapper 输入类型
- **低**：现有 302 行实现的 cancel/drain/join 协议不变

---

### 方案 B：多 sheet 并发

**核心思路**：每个 worksheet 使用独立的解析线程，包含完整的
XML 解析 → 行装配 → typed 转换 → invoke 流程。

**架构**：

```
workbook:
  sheet_0.xml ──▶ thread_0: full pipeline ──▶ listener.invoke()
  sheet_1.xml ──▶ thread_1: full pipeline ──▶ listener.invoke()
  sheet_N.xml ──▶ thread_N: full pipeline ──▶ listener.invoke()
```

**适用场景**：

- workbook 包含多个独立 sheet，每个 sheet 的处理互不依赖
- 用户需要 `all_sheets()` 或显式选择多个 sheet

**实现路径**：

1. `read_xlsx_source_with_consumer`（`read_xlsx.rs:62-105`）当前是
   串行遍历 `names` 列表。改为 `std::thread::scope` 或 rayon
   并行迭代。
2. 每个线程独立打开 `XlsxSource` → `XlsxRowMetadata` →
   `XlsxCellEventReader`，因为 OOXML ZIP 条目需要独立的 `Read + Seek`。
3. 需要共享 `ReadOptions`（已是不可变引用）和 `SharedStringCache`
   （已是 `Arc`-based）。
4. 下游 `ReadListener` 回调需要同步（`Mutex` 或每个 sheet 独立
   listener 实例后合并结果）。

**优势**：

- 对多 sheet 场景有真正的吞吐提升
- 每个 sheet 的 XML 解析完全独立，无共享状态

**劣势**：

- **对 benchmark 单 sheet 场景无收益**：benchmark 只读一个 sheet
- **复杂度高**：需要处理 OOXML ZIP 的并发读取、shared strings
  的线程安全、listener 回调的有序性保证
- **API 变更**：`ReadListener` trait 假设单线程回调，并发 sheet
  需要引入 `Send + Sync` 约束或新的 listener wrapper
- **内存开销**：每个 sheet 线程独立持有 XML 缓冲区、shared string
  cache 副本

**风险**：

- **高**：改变 listener 回调的有序性假设，影响所有下游 listener
- **高**：OOXML ZIP 并发读取的正确性验证复杂
- **中**：shared strings cache 的线程安全需要额外 `Arc<RwLock<>>` 开销

---

### 方案 C：流水线 parse → transform → commit

**核心思路**：将读取管线拆分为三个阶段，每个阶段运行在独立线程上，
通过有界通道连接。

**架构**：

```
Stage 1 (parse):     XML 解析 → 输出 {cells, metadata, seq}
        │  bounded channel (capacity C1)
        ▼
Stage 2 (transform): RowData::from_stream_parts + T::from_row_with_converters
        │  bounded channel (capacity C2)
        ▼
Stage 3 (commit):    downstream.invoke() 有序提交
```

**瓶颈分析**：

| 阶段 | Benchmark 场景 | 真实业务场景 |
|------|---------------|-------------|
| Stage 1 (parse) | XML 解析 + cell 收集 | XML 解析 + cell 收集 |
| Stage 2 (transform) | `apply_benchmark_map`（32x 哈希） | `from_row_with_converters`（converter 链） |
| Stage 3 (commit) | `invoke` 累加 checksum | `invoke` 业务处理 |

- **Benchmark**：Stage 2 是瓶颈（32x 哈希 >> parse + commit）。
  三阶段流水线的收益来自 overlap：当 Stage 2 处理 row N 时，
  Stage 1 可以同时解析 row N+1。
- **真实业务**：Stage 2 仍是瓶颈（converter 链比 XML 解析更重）。
  流水线收益与方案 A 类似，但复杂度显著更高。

**实现路径**：

1. Stage 1 线程：持有 `XlsxCellEventReader`，解析一行后通过
   `SyncSender<(u64, Vec<CellValue>, SourceRowMetadata)>` 发送。
2. Stage 2 线程池（N 个 worker）：接收行数据，执行
   `RowData::from_stream_parts` + `T::from_row_with_converters`，
   通过 `SyncSender<(u64, Result<T>, AnalysisContext)>` 发送。
3. Stage 3 线程（或主线程）：BTreeMap 按序接收，调用
   `downstream.invoke()`。

**优势**：

- 理论上最大并行度：三个阶段可以 overlap
- 对 XML 解析较重（大 sheet、复杂格式）且 mapper 较轻的场景有收益

**劣势**：

- **复杂度最高**：三个阶段的错误传播、取消协议、背压协调
- **与方案 A 收益相近**：当 Stage 2 是瓶颈时，Stage 1/3 的
  overlap 收益有限（Stage 2 的 worker 数已经决定了并行度）
- **额外延迟**：两个 channel hop 增加了每行的固定延迟
- **内存开销**：两个 channel 的缓冲区 + Stage 2 worker 的
  RowData 临时分配
- **benchmark 收益不确定**：当前方案 A 已满足 1.20x gate，
  方案 C 的额外 Stage 1/3 overlap 可能只带来 5-15% 的边际提升

**风险**：

- **高**：三阶段取消协议的正确性验证（当前单阶段 cancel/drain
  已有 100+ 行逻辑）
- **高**：Stage 1 XML 解析错误需要传播到 Stage 2/3 并触发
  有序排空
- **中**：两个 channel 的背压协调可能导致死锁（当前方案的
  `sync_channel` + `channel` 组合已经需要仔细设计）

---

## 3. 方案对比总结

| 维度 | 方案 A（保单解析，并发转换） | 方案 B（多 sheet 并发） | 方案 C（三阶段流水线） |
|------|---------------------------|----------------------|---------------------|
| 复杂度 | 低（现有架构扩展） | 高（并发 ZIP 读取 + listener 同步） | 高（三阶段协议 + 双通道） |
| Benchmark 收益 | 已满足 1.20x gate | 无（单 sheet） | 边际（~5-15% 超过 A） |
| 真实场景收益 | 中（mapper 并发） | 高（多 sheet） | 中（overlap 有限） |
| API 变更 | 无 | 需要 `Send + Sync` 约束 | 无（内部实现） |
| 风险 | 低 | 高 | 高 |
| 实现工时 | 4h 调研 + 16h 实现 | 24h+ | 20h+ |
| 与现有架构兼容 | 完全兼容 | 需要重构 read_xlsx_source | 兼容（内部重构） |

---

## 4. 推荐方案

**推荐方案 A**（保单 XML 解析，并发"行→typed row"转换）。

理由：

1. **最低风险**：不改变 XML 解析架构，不引入新的并发原语，
   现有 302 行 `ParallelMapReadListener` 的 cancel/drain/join
   协议完全复用。
2. **最高 ROI**：已满足 `internal_parallel_map` gate（2 worker
   speedup >= 1.20x，`benchmark-suite-v1.json:28`），无需额外工作。
3. **与现有架构完全兼容**：`ParallelMapReadListener` 作为
   `ReadListener<T>` 的 decorator，对上游（XML 解析）和下游
   （用户 listener）都是透明的。
4. **真实场景有扩展空间**：如果未来需要将 `RowData::from_stream_parts`
   + `T::from_row_with_converters` 也并发化，只需扩展 mapper
   的输入类型，不需要改变并发模型。

方案 B/C 留作长期选项：

- **方案 B** 在 workbook 有多个大 sheet 时有真正价值，但需要
  重构 `read_xlsx_source` 的 sheet 遍历逻辑和 listener 同步机制。
- **方案 C** 在 XML 解析成为瓶颈时（如超大 sheet、复杂格式）
  有理论优势，但当前 benchmark 和真实场景的瓶颈都在 mapper。

---

## 5. 影响分析

### 5.1 对现有测试的影响

**方案 A（推荐）不改变任何公共 API，对现有测试零影响。**

- **8 个 ExcelRows 测试**（`read/tests_cases/`）：不受影响。
  `ParallelMapReadListener` 是 opt-in 的 listener wrapper，
  不改变 `read_xlsx` 的默认行为。
- **35 个 web conformance 测试**：不受影响。这些测试使用默认
  `ReadListener`，不涉及 `ParallelMapReadListener`。
- **`ParallelMapReadListener` 自身的单元测试**：现有测试覆盖
  cancel/drain/join 协议，不需要新增。

### 5.2 新参数评估

**方案 A 不需要新参数。**

现有参数已经足够：

- `worker_count: usize`：worker 线程数（benchmark 用 1/2/4）
- `queue_capacity: usize`：有界队列容量（benchmark 用 64*worker_count）
- `mapper: Fn(T, &AnalysisContext) -> Result<U>`：用户定义的转换函数

如果未来扩展 mapper 输入类型（方案 A 的扩展方向），可以考虑：

- `parallel_depth: usize`：控制并发阶段数（1 = 仅 mapper，
  2 = mapper + RowData 构造）。但这是 P2+ 的优化，当前不需要。

### 5.3 对 `internal_parallel_map` gate 的影响

方案 A 不改变 `ParallelMapReadListener` 的并发语义，因此：

- `benchmark-suite-v1.json:27-35` 的 gate 定义不变
- `run_matrix.py:539-596` 的 `run_internal_parallel_map` 不变
- `compare_results.py:1165-1213` 的 `enforce_internal_parallel_map_gate` 不变
- 2 worker speedup >= 1.20x 已满足（基于 benchmark 的 32x 哈希 mapper）

---

## 6. 风险清单

| 风险 | 方案 | 影响 | 缓解措施 |
|------|------|------|---------|
| XML 解析仍是单线程瓶颈 | A/B/C | 对超大 sheet 的吞吐有上限 | 当前 XLSX 格式限制；无法在不预扫描的情况下并行化单 XML 流 |
| mapper panic 导致 sequence 丢失 | A（已有） | 主线程等待永不返回的结果 | 已有 `catch_unwind` + cancel 协议（`:127-135`） |
| cancel 期间 drain/join 死锁 | A（已有） | 管线挂起 | 已有 `SyncSender` + `channel` 组合设计（`:79-81`） |
| 方案 B 的 listener 线程安全 | B | 数据竞争 | 需要 `Mutex<dyn ReadListener>` 或每 sheet 独立 listener |
| 方案 C 的三阶段取消协议 | C | 错误传播不完整 | 需要重新设计 cancel/drain 逻辑（当前 100+ 行） |

---

## 7. 结论

**方案 A 是唯一推荐的短期方案**。它已经满足 `internal_parallel_map`
gate 的 1.20x speedup 要求，不需要改变任何公共 API 或并发模型，
对现有测试零影响。方案 B/C 的额外复杂度在当前场景下无法带来
显著收益，留作长期选项。

如果未来需要进一步提升并发收益，建议的优先级是：

1. **方案 A 扩展**：将 `RowData::from_stream_parts` +
   `T::from_row_with_converters` 纳入 mapper（4h 实现）
2. **方案 B**：多 sheet 并发（24h+ 实现，需要 API 变更）
3. **方案 C**：三阶段流水线（20h+ 实现，收益不确定）
